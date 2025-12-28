mod helper;
mod drone_interface;
mod error;
//#[cfg(debug_assertions)]
pub(crate) mod debug_utils;

mod video;
pub(crate) mod logger;
mod app_network;

// TODO: Figure out if this is a fair concession until we fully implement the features.
//		Not to mention, these concerns about unused imports and variables are just plainly wrong.
#[allow(dead_code, unused_imports)]
mod database;

#[cfg(test)]
mod tests;
use crate::app_network::{send_image, ConnectionState, InfoPacket, RoShamBo, VideoCode};
use crate::app_network::{handle_connection, handle_control_activity, handle_info_activity, ClientSocketType};
use crate::drone_interface::Drone;
use crate::logger::{do_logging, Logger};
use error::Error;
use local_ip_address::local_ip;
use mio;
use mio::event::Event;
use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Events, Interest, Poll, Token, Waker};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};
use mio_wakeq::{WakeQ, WakeQSender};
use takeflight_computer_vision as computer_vision;
use video::video_queue::VideoQueue;
use crate::video::video_queue::VideoTaskFull;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Connection
{
	TCP(TcpStream),
	UDP(UdpSocket),
	// FIXME: All of these should have distinct ClientSocketType...
	ClientControl(ClientSocketType, Arc<Mutex<TcpStream>>),
	VideoOut(ClientSocketType, Arc<Mutex<TcpStream>>), // This one is for sending video to the client. There will be a "VideoIn," which will be used for the CV pipeline.
	ServerInfo(ClientSocketType, Arc<Mutex<TcpStream>>),
	Drone(Arc<Mutex<dyn Drone>>),
	Camera(), // FIXME: This needs fields.
}

#[derive(Debug)]
#[repr(usize)]
pub(crate) enum InternalSignal
{
	PingEveryone,
	//ToVideoQueue(VideoTaskFull),		// I don't think this will be used.
	FromVideoQueue((Token, Box<[u8]>)),	// The token will always be a "Video Source," no refactoring it out.
}


type ServerMap = Arc<Mutex<HashMap<Token, Connection>>>;

pub(crate) const LISTENER			: Token = Token(0);	// 0 is the reserved file descriptor for stdin. It cannot be used for ports, so listener is always valid.
const TOKEN_START: usize = u16::MAX as usize;
pub(crate) const INTERNAL_SIGNALLER	: Token = Token(TOKEN_START + 1 ); // 1 is reserved by the system for stdout. (2 is stdout, we can use it as well.)

const LOG_DIR : &str = "logs/";
const TIMEOUT : Duration = Duration::from_millis((1.5 * 1000.0) as u64);

// TODO: Let's make a more accurate name for this.
#[allow(dead_code)]
struct ServerInstance
{
	// video_stream : //TODO: let's make an enum where it can be a drone, or a separate TCP
	listener		: TcpListener,
	ownership_map	: Arc<Mutex<HashMap<Token, Connection>>>,
	poll 			: Arc<Mutex<Poll>>,
	logger			: Logger,
	video_out		: Arc<Mutex<Option<Token>>>,	// If this connection is not found in the map, this will be set to None. This should not be accessed directly.
	video_src		: Arc<Mutex<Option<Token>>>,	// If this connection is not found in the map, this will be set to None. This should not be accessed directly.
	drone_control	: Option<Token>,				// If this is None, we will travel the whole map and send signals to every found drone.
	info_token		: Option<Token>,				// If this is None, we will travel the whole map and send signals to every found drone.
	ro_sham_bo		: RoShamBo,
	frame_time		: Arc<Duration>,
	read_buffer		: [u8;1024],
	curr_drone		: Option<Arc<Mutex<Connection>>>,
	internal_signal_receiver : Rc<WakeQ<InternalSignal>>
}

const HEARTBEAT_TIME: Duration = Duration::from_millis(400);
//Main fn that executes the application within a localhost http with the return signature Result<(), Error>
//Allowing for proper error handling in case the application can not be opened
fn main() -> Result<(), Error> {
	const FRAME_TIME: Duration = Duration::from_millis(1000 / 20); // 20 fps doesn't seem bad for now.

	let mut continue_logger = Arc::new(Mutex::new(true));
	let mut continue_heartbeat = Arc::new(Mutex::new(true));

	// TODO: Add logic for determining log file.
	let log_file = "log_file";
	std::fs::create_dir(LOG_DIR).unwrap_or_default(); // Make sure the file directory exists
	let file = Arc::new(Mutex::new(Some(File::create(format!("{LOG_DIR}{log_file}"))?)));
	let (logger, receiver) = logger::Logger::new();

	// Start the logger
	let logger_handle = {
		let cloned_file = file.clone();
		let cloned_continue_logger = continue_logger.clone();
		thread::Builder::new()
			.name(String::from("Logger"))
			.spawn(move || { do_logging(receiver, cloned_file, cloned_continue_logger).unwrap()
			})?
	};
	logger.info("Logger started!")?;

	// Start the server
	let poll = Arc::new(Mutex::new(Poll::new()?));
	let mut listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
	poll.lock()?.registry().register(&mut listener, LISTENER, Interest::READABLE)?;
	let server_address = listener.local_addr()?;

	// Handle arguments
	{
		let mut args = std::env::args();
		logger.info_from_string(format!("We received {} arguments...", args.len()))?;
		if args.len() == 2
		{
			let port : u16 = args.nth(1).unwrap().parse()?;
			let port_to_send = server_address.port();
			let negotiator = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			let client_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
			negotiator.send_to(&port_to_send.to_be_bytes(), client_address)?;
			logger.info_from_string(format!("Sent port {} to {}:{}", port_to_send, client_address, client_address.port()))?;
		}
	}

	//test
	logger.info(&format!("Listening on all IPv4 interfaces. Network address: {}, port {}", local_ip()?, server_address.port()))?;

	// Get the internal signaller set up
	let internal_signal_receiver = WakeQ::new(&*poll.lock()?, INTERNAL_SIGNALLER)?;

	// Start the video queue
	let video_src = Arc::new(Mutex::new(None));
	let (queue_sender, video_handle) = VideoQueue::start_work_thread(&*poll.lock()?, video_src.clone(), logger.clone(), internal_signal_receiver.get_sender())?;

	// Start heartbeat
	let heartbeat_handle = do_heartbeat(continue_heartbeat.clone(), logger.clone(), internal_signal_receiver.get_sender())?;


	// We will be implementing the TakeFlight server backend here. Since the process is spawned we can do our anything here
	let ownership_map = Arc::new(Mutex::new(HashMap::<Token, Connection>::new()));

	logger.info("Server starting!!!")?;

	let mut server = ServerInstance {
		listener,
		ownership_map,
		poll,
		logger 						: logger.clone(),
		video_src,
		video_out					: Arc::new(Mutex::new(None)),
		drone_control				: None,
		info_token					: None,
		ro_sham_bo					: RoShamBo::Rock,
		frame_time					: Arc::new(FRAME_TIME),
		read_buffer					: [0;1024],
		curr_drone					: None,
		internal_signal_receiver	: internal_signal_receiver.into(),
	};

	// test
	//let drone = crate::drone_interface::drone_pro::Drone::new(server.poll.clone(), server.ownership_map.clone(), server.logger.clone(), server.video_src.clone(), server.video_out.clone(), server.frame_time.clone())?;
	let drone = crate::drone_interface::tello::drone::TelloDrone::new(server.poll.clone(), server.ownership_map.clone(), server.logger.clone(), server.video_src.clone(), server.video_out.clone(), server.frame_time.clone(), queue_sender.clone())?;
	server.curr_drone = Some(Arc::new(Mutex::new(Connection::Drone(drone.clone()))));
	/*drone.lock()?.takeoff()?;
	sleep(Duration::from_secs(5));
	drone.lock()?.rc(0, 99, 0, 0.0)?;
	sleep(Duration::from_secs(5));
	print!("\x07");
	drone.lock()?.graceful_land()?;
	 */

	// Some multiplexing
	match server.multiplex()
	{
		Ok(_) => { /* noop */ }
		Err(e) => { logger.error_from_string(format!("Server an encountered error, shutting down:\n\"\"\"\n{e}\n\"\"\""))? }
	}

	try_join(heartbeat_handle, &mut continue_heartbeat)?;
	shutdown_video(video_handle, queue_sender)?;

	logger.info("Server shutting down.")?;
	try_join(logger_handle, &mut continue_logger)?;

	Ok(())
} // main


impl ServerInstance
{
	fn multiplex(&mut self) -> Result<(), Error>
	{
		// While I don't like the idea of this being a local variable,
		// it only makes sense given that event_buffer is used in precisely
		// two places. The poll stage, and the top of the drain stage.
		const MAX_EVENTS : usize = 1024;
		let mut event_buffer = Events::with_capacity(MAX_EVENTS);

		loop
		{
			// Receive and handle events
			self.poll.lock()?.poll(&mut event_buffer, None)?;
			let events_result = self.drain_events(&mut event_buffer);

			if events_result.is_ok() { continue }
			let return_error = events_result.err().unwrap();

			match &return_error
			{
				Error::IOError(e) => {
					match e.kind()
					{
						ErrorKind::WouldBlock => { continue }
						_ => break Err(return_error)
					}
				}

				_ => { break Err(return_error) }
			}

		}
	}

	fn drain_events(&mut self, event_buffer : &mut Events) -> Result<(), Error>
	{

		for event in event_buffer.iter()
		{
			// TODO: If one drone socket disconnects, do we want to let the drone decide what to do?
			if Event::is_read_closed(event)
			{
				self.disconnect_client(&event);
				continue
			}


			match event.token()
			{
				LISTENER => { self.handle_listener_events() }
				//HEARTBEAT => { self.handle_heartbeat_events() }
				//VIDEO_QUEUE => { self.handle_video_queue_events() }
				INTERNAL_SIGNALLER => {
					for internal_signal in self.internal_signal_receiver.clone().iter_pending_events()
					{
						match internal_signal
						{
							InternalSignal::PingEveryone => { self.handle_heartbeat_events() }
							InternalSignal::FromVideoQueue(event) => { self.handle_video_queue_events(event) }
						}?
					}

					Ok(())
				}
				token => { self.handle_token(token) }
				// It's assuring for the compiler to prove that this condition is unreachable.
				#[allow(unreachable_patterns)]
				_ => { todo!("What is this?") }
			}?
		}

		Ok(())
	} // fn drain_events

	/// If Some(true), `continue`
	/// If Some(false), keep executing.
	fn handle_token(&mut self, token : Token) -> Result<(), Error>
	{

		// This is gore.
		// We need to remove the silly TCP stream to reassign it to a proper role. I desperately need to find a better way.
		let cloned_connection = {
			let ownership_map_clone = self.ownership_map.clone(); // Avoids a partial borrow which stops borrowing the whole object later, doesn't really add overhead.
			let mut ownership_map_lock = ownership_map_clone.lock()?;

			let needs_reassigned = {
				let found_connection_wrapped = ownership_map_lock.get(&token);
				if found_connection_wrapped.is_none()
				{
					self.logger.error_from_string(format!("Unmapped port: {}", token.0))?;
					return Err(Error::Custom("Somehow registry included an unmapped value! Shutting down server!"));
				}

				match found_connection_wrapped {
					Some(Connection::TCP(_)) => { true }
					_ => { false }
				}
			};

			if needs_reassigned
			{
				let found_connection = ownership_map_lock.remove(&token);
				match found_connection
				{
					// Handle the connection.
					Some(Connection::TCP(stream)) => { handle_connection(stream, self, &mut *ownership_map_lock)?}
					_ => { return Ok(()) }
				};

				// CLARIFY:
				// 	This logic should be handled in the above function now.
				//ownership_map_lock.insert(token, new_item);
				return Ok(());
			}

			let found_connection = ownership_map_lock.get(&token);
			found_connection.unwrap().try_clone()
		};

		#[cfg(debug_assertions)]	// I wanna keep the logs fairly light in release.
		self.logger.info("Sending out keep-alives! FIXME: This is not actually the proper place for keep-alive, I genuinely have no clue how this got here.")?;
		// CLARIFY: It's not clear right now if it's necessary to check the unwrap of this one, on the grounds that non-cloneables should be caught in the needs_reassigned block.
		match cloned_connection
		{
			Some(found) => {
				match found {
					Connection::ServerInfo(..) => { handle_info_activity(token, self, &mut *self.ownership_map.clone().lock()?)?; }
					Connection::Drone(drone) => {
						// Receive signal will always go until an error is encountered.
						// Below is the pattern matching for that error. We can recover from WouldBlock, but there are many layers of indirection.
						match drone.lock()?.receive_signal(token.0 as u16) {
							Err(e) => {
								match e
								{
									Error::IOError(io_error) => {
										if io_error.kind() == ErrorKind::WouldBlock { /* noop */ } else { Err(io_error)? }
									}
									_ => { Err(e)? }
								}
							}
							_ => { /* noop */ }
						}
					}
					Connection::ClientControl(..) => { handle_control_activity(token, self, &mut *self.ownership_map.clone().lock()?)? }
					_ => { Err(Error::Custom("Error within drain_events token case. Did not know how to handle this connection..."))? }
				};
			}
			None => panic!("We already checked that the connection exists.")
		}

		Ok(())
}


	/*fn send_image(&mut self) -> Result<(), Error>
	{
		// The only pessimization to this wrapper is the arc increment
		send_image(self.video_out.clone(), self.video_src.clone(), self.ownership_map.clone())
	}*/

	/// Doesn't throw error if there is no info socket. Instead, it's a noop.
	/// FIXME: I want to do a little less passing in of ownership maps...
	fn send_info(&mut self, packet : &InfoPacket, ownership_map: &HashMap<Token, Connection>) -> Result<(), Error>
	{
		match &self.info_token
		{
			// If there's no token, that's fine, nothing to be done.
			None => {
				self.logger.warn("Tried to send an info message out, but there was no candidate")
			}

			Some(token) => {
				match ownership_map.get(token)
				{
					// If the token doesn't actually exist, that's a problem. But one that we can recover from!
					None => {
						self.info_token = None; // I guess this is valid because Token is copy?
						Err(Error::Custom("While attempting to send info, found that the current token was invalid!"))
					}

					Some(connection) => {
						match connection
						{
							// The nominal case. We can send the dang message.
							Connection::ServerInfo(_, stream) => {
								let mut stream_lock = stream.lock()?;
								packet.write(&mut *stream_lock)
							}

							// If the server is the wrong type, that's a problem, but one we can also recover from!
							_ => {
								self.info_token = None;
								Err(Error::Custom("While attempting to send info, found that the info socket was the wrong type of connection!"))
							}
						}
					}
				} // match ownership_map.get(token)
			} // Some(token)
		} // match &self.info_token
	} // fn send_info


	fn disconnect_client(&mut self, event : &Event) -> Result<(), Error>
	{
		let token = event.token();
		let socket_wrapped = self.ownership_map.lock()?.remove(&token);
		match socket_wrapped {
			Some(mut socket) => {
				match &mut socket {
					Connection::TCP(stream) => { self.poll.lock()?.registry().deregister(stream)?; }
					Connection::UDP(datagram) => {self.poll.lock()?.registry().deregister(datagram)?; }
					Connection::ClientControl(_, stream) => { self.poll.lock()?.registry().deregister(&mut *stream.lock()?)?; }
					Connection::VideoOut(_, stream) => { self.poll.lock()?.registry().deregister(&mut *stream.lock()?)?; }
					Connection::ServerInfo(_, stream) => { self.poll.lock()?.registry().deregister(&mut *stream.lock()?)?; }
					Connection::Drone(drone) => { todo!("We definitely want to let the drone decide what to do here. Take note that the ownership map is NOT locked right now. {:?}", drone) }
					Connection::Camera() => { todo!("We still do not have support for cameras yet.") }
				};

				self.logger.info_from_string(format!("Disconnected {} socket: {}", socket.socket_type_name(), event.token().0))?;
			}
			None => { todo!("Somehow we received a 'read closed' event for a socket that isn't registered to the poll... This shouldn't be possible. Please try to reproduce this issue.") }
		}

		Ok(())
	} // fn disconnect_client

	fn handle_listener_events(&mut self) -> Result<(), Error>
	{
		// Accept all incoming streams.
		loop {
			let incoming = self.listener.accept();
			match incoming
			{
				Ok((mut stream, address)) => {
					let token = Token(address.port() as usize);
					self.poll.lock()?.registry().register(
						&mut stream,
						token.clone(),
						Interest::READABLE)?;

					self.logger.info_from_string(format!("Accepting TCP stream: {}:{}", address.ip(), address.port()))?;

					self.ownership_map.lock()?.insert(
						token,
						Connection::TCP(stream),
					);
				}
				Err(e) => {
					if e.kind() == ErrorKind::WouldBlock { break Ok(()) }
					else { return Err(e.into()) }
				}
			}
		}
	} // fn handle_listener_events

	fn handle_heartbeat_events(&mut self) -> Result<(), Error>
	{
		// Send heartbeat to all eligible connections
		let mut contacted_drones : Vec<Arc<Mutex<dyn Drone + 'static>>> = Vec::new();
		let mut delete_drones : Vec<Arc<Mutex<dyn Drone + 'static>>> = Vec::new();
		let ownership_map_clone = self.ownership_map.clone();
		let mut ownership_map_lock = ownership_map_clone.lock()?;
		for connection in ownership_map_lock.iter() {
			// This seems like a patchy solution. This combats sending multiple pings per cycle.
			match connection.1
			{
				// TODO: This is sorely in need of a refactor...
				Connection::Drone(drone) => {
					if contacted_drones.iter().find(|ptr| { Arc::ptr_eq(ptr, drone) }).is_some() {
						continue
					}
					else {
						contacted_drones.push(drone.clone())
					}

					let mut drone_lock = drone.lock()?;
					if !drone_lock.connected()
					{
						let now = SystemTime::now();
						if now.duration_since(drone_lock.time_created())? > TIMEOUT
						{
							self.logger.error("Failed to connect to drone!!!")?;
							// CLARIFY: This should not be all 0's. This should be an actual MAC address.
							let state_packet = InfoPacket::new_drone_connection_state(self.ro_sham_bo.post_increment(),
																					  ConnectionState::FailedConnect,
																					  [0, 0, 0, 0, 0, 0],
							);

							self.send_info(&state_packet, &ownership_map_lock)?;
							delete_drones.push(drone.clone());
						}
						continue;
					}

					let ping_result = drone_lock.send_heartbeat();
					match ping_result {
						Ok(_)	=> { continue; }
						Err(e)	=> {
							match e {
								Error::IOError(io_error) => {
									if io_error.kind() == std::io::ErrorKind::WouldBlock {
										continue;
									}
									else {
										return Err(Error::Custom("IO error occurred while pinging a drone!"));
									}
								}
								_ => { return Err(Error::Custom("Generic error occurred while pinging a drone!")); }
							}
						}
					}
				}
				_ => { /* noop. TCP automatically sends pings, UDP doesn't have enough information to keep alive. */ }
			}
		}

		delete_drones.dedup_by(|left, right| {Arc::ptr_eq(&left, right)});
		for drone in delete_drones
		{
			// CLARIFY: This should not be all 0's. This should be an actual MAC address.
			let state_packet = InfoPacket::new_drone_connection_state(self.ro_sham_bo.post_increment(),
																	  ConnectionState::Disconnected,
																	  [0, 0, 0, 0, 0, 0],
			);
			self.send_info(&state_packet, &ownership_map_lock)?;
			drone.lock()?.disconnect(&mut ownership_map_lock)?;
		}

		Ok(())
	} // fn handle_heartbeat_events

	/// This function may have no effect if it receives an event that is no longer relevant.
	/// Such cases may happen when switching video sources.
	fn handle_video_queue_events(&mut self, video_event : (Token, Box<[u8]>)) -> Result<(), Error>
	{
		let internal_signal = self.internal_signal_receiver.iter_pending_events().nth(0).unwrap();
		let message = {
			if let InternalSignal::FromVideoQueue(message) = internal_signal { message }
			else { return Err(Error::Custom("Somehow a non-video_queue event ended up here...")) } // FIXME: I'm not sure that this is right...
		};
		todo!("Erm, haven't gotten that far yet...");
		// FIXME: This should not be hardcoded.
		let ownership_lock = self.ownership_map.lock()?;
		let token = match &*self.video_out.lock()? {
			Some(token) => { token.clone() }
			None => { return Ok(()) }
		};

		let out_sock = match ownership_lock.get(&token)
		{
			Some(sock) => { sock }
			None => { return Ok(()) }
		};

		match out_sock {
			Connection::VideoOut(_, stream) => {
				let mut stream_lock = stream.lock()?;
				// FIXME: this should not require additional allocations.
				let mut out_buffer = Vec::new();
				out_buffer.write(&(VideoCode::Png as u8).to_be_bytes())?;
				out_buffer.write(&(message.1.len() as u16).to_be_bytes())?;
				out_buffer.write(&(message.1))?;
				stream_lock.write_all(&out_buffer)?;
			}
			_ => todo!("Why is our video destination NOT a VideoOut????")
		}

		Ok(())
	} // fn handle_video_queue_events


} // impl ServerInstance



impl Connection
{
	pub(crate) fn try_clone(&self) -> Option<Self>
	{
		match self {
			Connection::ClientControl(client_type, stream_arc_mtx) => { Some(Connection::ClientControl(client_type.clone(), stream_arc_mtx.clone())) }
			Connection::VideoOut(client_type, stream_arc_mtx) => { Some(Connection::VideoOut(client_type.clone(), stream_arc_mtx.clone())) }
			Connection::ServerInfo(client_type, stream_arc_mtx) => { Some(Connection::ServerInfo(client_type.clone(), stream_arc_mtx.clone())) }
			Connection::Drone(drone) => { Some(Connection::Drone(drone.clone())) }
			Connection::Camera() => { todo!("We have no support yet for camera types. We're not sure if cloning a camera is even possible.") }
			_ => {	None }
		}
	}

	pub(crate) const fn socket_type_name(&self) -> &'static str
	{
		match self {
			Connection::TCP(..)				=> { "Unpromoted TCP" }
			Connection::UDP(..)				=> { "Unpromoted UDP" }
			Connection::ClientControl(..)	=> { "ClientControl" }
			Connection::VideoOut(..)		=> { "VideoOut" }
			Connection::ServerInfo(..)		=> { "ServerInfo" }
			Connection::Drone(..)			=> { "Drone" }
			Connection::Camera(..)			=> { "Camera" }
		}
	}


}

impl TryInto<TcpStream> for Connection
{
	type Error = crate::Error;

	fn try_into(self) -> Result<TcpStream, Self::Error> {
		match self {
			Connection::TCP(stream) => Ok(stream),
			_ => Err(Error::Custom("Not a TCP stream buddy...")),
		}
	}
}


fn try_join<T>(join_handle: JoinHandle<T>, predicate : &mut Arc<Mutex<bool>>) -> Result<(), Error>
{
	let thread_name = join_handle.thread().name().unwrap_or_default().to_string();
	match predicate.lock()
	{
		Ok(mut predicate_inner) => { *predicate_inner = !*predicate_inner; }
		// It's OK if this thread crashed, we're only trying to close out the threads.
		Err(_) => { println!("It appears that thread \"{}\" had a poisoned mutex. It likely crashed.", thread_name); }
	} // match predicate

	// This is fine due to the errors this returns
	// If the thread panics, it bubbles the error -- we're shutting down anyway
	match join_handle.join() {
		Ok(_) => { println!("Successfully closed thread \"{}\"!", thread_name) }
		Err(e) => { println!("There may have been an error closing thread \"{}\". {:?}", thread_name, e)}
	}

	Ok(())
}


// FIXME, make this safer???
fn do_heartbeat(continue_heartbeat : Arc<Mutex<bool>>, logger : Logger, sender : WakeQSender<InternalSignal>) -> Result<JoinHandle<()>, Error>
{
	Ok(thread::Builder::new()
		.name("Heart".into())
		.spawn(|| heartbeat_entrypoint(sender, continue_heartbeat, logger).unwrap())?)
}

fn heartbeat_entrypoint(sender : WakeQSender<InternalSignal>, continue_heartbeat : Arc<Mutex<bool>>, logger : Logger) -> Result<(), Error>
{
	while *continue_heartbeat.lock().unwrap()
	{
		thread::sleep(crate::HEARTBEAT_TIME);
		sender.send_event(InternalSignal::PingEveryone)?; // No shot this fails, but if it does, we don't care anyway.
	}

	logger.warn("Shutting down!")
}


// FIXME: make this safer???
fn shutdown_video<T>(join_handle: JoinHandle<T>, queue_sender : VideoQueue) -> Result<(), Error>
{
	let thread_name = join_handle.thread().name().unwrap_or_default().to_string();
	match queue_sender.shutdown()
	{
		Ok(_) => { println!("Sent shutdown signal to video queue.") }
		Err(_) => { println!("Encountered an error sending shutdown signal to video queue. Continuing.") }
	}
	match join_handle.join() {
		Ok(_) => { println!("Successfully closed thread \"{}\"!", thread_name) }
		Err(e) => { println!("There may have been an error closing thread \"{}\", or the thread already closed.", thread_name)}
	}

	Ok(())
}