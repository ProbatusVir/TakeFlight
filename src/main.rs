mod helper;
mod video_stream;
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

use crate::app_network::{handle_connection, handle_info_activity, ClientSocketType};
use crate::drone_interface::{ConnectionState, Drone};
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
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use takeflight_computer_vision as computer_vision;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Connection
{
	TCP(TcpStream),
	UDP(UdpSocket),
	ClientControl(ClientSocketType, Arc<Mutex<TcpStream>>),
	VideoOut(ClientSocketType, Arc<Mutex<TcpStream>>), // This one is for sending video to the client. There will be a "VideoIn," which will be used for the CV pipeline.
	ServerInfo(ClientSocketType, Arc<Mutex<TcpStream>>),
	Drone(Arc<Mutex<dyn Drone>>),
	Camera(), // FIXME: This needs fields.
}

type ServerMap = Arc<Mutex<HashMap<Token, Connection>>>;

const LISTENER : Token = Token(0);	// 0 is the reserved file descriptor for stdin. It cannot be used for ports, so listener is always valid.
const HEARTBEAT : Token = Token(1); // 1 is reserved by the system for stdout. (2 is stdout, we can use it as well.)
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
	frame_time		: Arc<Duration>,
	read_buffer		: [u8;1024],
}

//Main fn that executes the application within a localhost http with the return signature Result<(), Error>
//Allowing for proper error handling in case the application can not be opened
fn main() -> Result<(), Error> {
	const MAX_EVENTS : usize = 1024;
	const HEARTBEAT_TIME: Duration = Duration::from_millis(400);
	const FRAME_TIME: Duration = Duration::from_millis(1000 / 20); // 20 fps doesn't seem bad for now.

	// TODO: Add logic for determining log file.
	let log_file = "log_file";
	std::fs::create_dir(LOG_DIR).unwrap_or_default(); // Make sure the file directory exists
	let file = Arc::new(Mutex::new(Some(File::create(format!("{LOG_DIR}{log_file}"))?)));
	let (logger, receiver) = logger::Logger::new();

	// Start the logger
	{
		let cloned_file = file.clone();
		thread::Builder::new()
			.name(String::from("Logger"))
			.spawn(move || { do_logging(receiver, cloned_file).unwrap()
			})?;
	}

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


	// Start heartbeat
	{
		let heartbeat = Waker::new(poll.lock()?.registry(), HEARTBEAT)?;
		thread::spawn(move || { loop {
			thread::sleep(HEARTBEAT_TIME);
			heartbeat.wake().unwrap_or(()); // No shot this fails, but if it does, we don't care anyway.
		} });
	}


	// We will be implementing the TakeFlight server backend here. Since the process is spawned we can do our anything here
	let ownership_map = Arc::new(Mutex::new(HashMap::<Token, Connection>::new()));
	let mut event_buffer = Events::with_capacity(MAX_EVENTS);

	logger.info("Server starting!!!")?;

	let mut server = ServerInstance {
		listener,
		ownership_map,
		poll,
		logger 			: logger.clone(),
		video_src		: Arc::new(Mutex::new(None)),
		video_out		: Arc::new(Mutex::new(None)),
		drone_control	: None,
		info_token		: None,
		frame_time		: Arc::new(FRAME_TIME),
		read_buffer		: [0;1024],
	};

	// test
	//let drone = crate::drone_interface::drone_pro::Drone::new(server.poll.clone(), server.ownership_map.clone(), server.logger.clone(), server.video_src.clone(), server.video_out.clone(), server.frame_time.clone())?;
	let drone = crate::drone_interface::tello::drone::TelloDrone::new(server.poll.clone(), server.ownership_map.clone(), server.logger.clone(), server.video_src.clone(), server.video_out.clone(), server.frame_time.clone())?;
	/*drone.lock()?.takeoff()?;
	sleep(Duration::from_secs(5));
	drone.lock()?.rc(0, 99, 0, 0.0)?;
	sleep(Duration::from_secs(5));
	print!("\x07");
	drone.lock()?.graceful_land()?;
	 */

	// Some multiplexing
	let status = loop
	{
		// Receive and handle events
		server.poll.lock()?.poll(&mut event_buffer, None)?;
		let events_result = drain_events(&mut server, &mut event_buffer, &logger);

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

	};

	status
}

fn drain_events(server: &mut ServerInstance, event_buffer : &mut Events, logger : &Logger)
				-> Result<(), Error>
{
	for event in event_buffer.iter()
	{
		// TODO: If one drone socket disconnects, do we want to let the drone decide what to do?
		if Event::is_read_closed(event)
		{
			let token = event.token();
			let socket_wrapped = server.ownership_map.lock()?.remove(&token);
			match socket_wrapped {
				Some(mut socket) => {
					match &mut socket {
						Connection::TCP(stream) => { server.poll.lock()?.registry().deregister(stream)?; }
						Connection::UDP(datagram) => {server.poll.lock()?.registry().deregister(datagram)?; }
						Connection::ClientControl(_, stream) => { server.poll.lock()?.registry().deregister(&mut *stream.lock()?)?; }
						Connection::VideoOut(_, stream) => { server.poll.lock()?.registry().deregister(&mut *stream.lock()?)?; }
						Connection::ServerInfo(_, stream) => { server.poll.lock()?.registry().deregister(&mut *stream.lock()?)?; }
						Connection::Drone(drone) => { todo!("We definitely want to let the drone decide what to do here. Take note that the ownership map is NOT locked right now. {:?}", drone) }
						Connection::Camera() => { todo!("We still do not have support for cameras yet.") }
					};

					server.logger.info_from_string(format!("Disconnected {} socket: {}", socket.socket_type_name(), event.token().0))?;
				}
				None => { todo!("Somehow we received a 'read closed' event for a socket that isn't registered to the poll... This shouldn't be possible. Please try to reproduce this issue.") }
			}
			continue
		}

		match event.token()
		{
			LISTENER => {
				// Accept all incoming streams.
				loop {
					let incoming = server.listener.accept();
					match incoming
					{
						Ok((mut stream, address)) => {
							let token = Token(address.port() as usize);
							server.poll.lock()?.registry().register(
								&mut stream,
								token.clone(),
								Interest::READABLE)?;

							server.logger.info_from_string(format!("Accepting TCP stream: {}:{}", address.ip(), address.port()))?;

							server.ownership_map.lock()?.insert(
								token,
								Connection::TCP(stream),
							);
						}
						Err(e) => {
							if e.kind() == ErrorKind::WouldBlock {break}
							else { return Err(e.into()) }
						}
					}
				}
			}
			HEARTBEAT => {
				// Send heartbeat to all eligible connections
				let mut contacted_drones : Vec<Arc<Mutex<dyn Drone + 'static>>> = Vec::new();
				let mut delete_drones : Vec<Arc<Mutex<dyn Drone + 'static>>> = Vec::new();
				let mut ownership_map_lock = server.ownership_map.lock()?;
				for connection in ownership_map_lock.iter_mut() {
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
							let drone_connection_state = drone_lock.connection_state();

							// Log the appropriate error, and send to client.
							match drone_connection_state {
								ConnectionState::FailedConnect => {
									logger.error("Failed to connect to drone!!!")?;
									delete_drones.push(drone.clone());
									continue;
								}
								ConnectionState::Disconnected => {
									logger.error("Drone disconnected.")?;
									delete_drones.push(drone.clone());
									continue;
								}
								_ => { /* noop */ }
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
					drone.lock()?.disconnect(&mut *ownership_map_lock)?;

				}
			}
			token => {

				// This is gore.
				// We need to remove the silly TCP stream to reassign it to a proper role. I desperately need to find a better way.
				let cloned_connection = {
					let ownership_map_clone = server.ownership_map.clone(); // Avoids a partial borrow which stops borrowing the whole object later, doesn't really add overhead.
					let mut ownership_map_lock = ownership_map_clone.lock()?;

					let needs_reassigned = {
						let found_connection_wrapped = ownership_map_lock.get(&token);
						if found_connection_wrapped.is_none()
						{
							logger.error_from_string(format!("Unmapped port: {}", token.0))?;
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
						let new_item = match found_connection
						{
							Some(Connection::TCP(stream)) => { handle_connection(stream, server)? }
							_ => { continue }
						};

						ownership_map_lock.insert(token, new_item);
						continue;
					}

					let found_connection = ownership_map_lock.get(&token);
					found_connection.unwrap().try_clone()
				};

				#[cfg(debug_assertions)]	// I wanna keep the logs fairly light in release.
				server.logger.info("Sending out keep-alives!")?;
				// CLARIFY: It's not clear right now if it's necessary to check the unwrap of this one, on the grounds that non-cloneables should be caught in the needs_reassigned block.
				match cloned_connection
				{
					Some(found) => {
						match found {
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
									_ => { panic!("This block should be unreachable.") }
								}
							}
							Connection::ServerInfo(..) => { handle_info_activity(token, server)? }
							_ => { Err(Error::Custom("Error within drain_events token case. Did not know how to handle this connection..."))? }
						};
					}
					None => panic!("We already checked that the connection exists.")
				}
			}
			// It's assuring for the compiler to prove that this condition is unreachable.
			#[allow(unreachable_patterns)]
			_ => { todo!("What is this?") }
		}
	}

	Ok(())
}


impl ServerInstance
{
	/*fn send_image(&mut self) -> Result<(), Error>
	{
		// The only pessimization to this wrapper is the arc increment
		send_image(self.video_out.clone(), self.video_src.clone(), self.ownership_map.clone())
	}*/

}

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



