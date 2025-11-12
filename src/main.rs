mod helper;
mod video_stream;
mod drone_interface;
mod error;
//#[cfg(debug_assertions)]
pub(crate) mod debug_utils;

mod video;
pub(crate) mod logger;
mod app_network;
mod database;

#[cfg(test)]
mod tests;

use crate::drone_interface::Drone;
use error::Error;
use mio;
use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Events, Interest, Poll, Token, Waker};
use std::collections::HashMap;
use std::fmt::{Debug, };
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use local_ip_address::local_ip;
use crate::app_network::{handle_connection, ClientSocketType};
use crate::logger::{do_logging, Logger};
use takeflight_computer_vision as computer_vision;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Connection
{
	TCP(TcpStream),
	UDP(UdpSocket),
	Client(ClientSocketType, TcpStream),
	VideoOut(ClientSocketType, TcpStream), // This one is for sending video to the client. There will be a "VideoIn," which will be used for the CV pipeline.
	Drone(Arc<Mutex<dyn Drone>>),
	Camera(), // FIXME: This needs fields.
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

const LISTENER : Token = Token(0);	// 0 is the reserved file descriptor for stdin. It cannot be used for ports, so listener is always valid.
const HEARTBEAT : Token = Token(1); // 1 is reserved by the system for stdout. (2 is stdout, we can use it as well.)
const VID_WAKER : Token = Token(2);
const LOG_DIR : &str = "logs/";

// TODO: Let's make a more accurate name for this.
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

}

//Main fn that executes the application within a localhost http with the return signature Result<(), Error>
//Allowing for proper error handling in case the application can not be opened
fn main() -> Result<(), Error> {
	const MAX_EVENTS : usize = 1024;
	const HEARTBEAT_TIME: Duration = Duration::from_millis(3000);
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
			.spawn(move || { do_logging(receiver, cloned_file).unwrap() })?;
	}

	logger.info("Logger started!")?;

	// Start the server
	let poll = Arc::new(Mutex::new(Poll::new()?));
	let mut listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
	let server_address = listener.local_addr()?;

	// Handle arguments
	{
		let mut args = std::env::args();
		if args.len() == 2
		{
			let port : u16 = args.nth(1).unwrap().parse()?;
			let negotiator = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			negotiator.send_to(&server_address.port().to_be_bytes(), SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)))?;
			logger.info("Client asked for server socket.")?;
		}
	}

	//test
	logger.info(&format!("Listening on all IPv4 interfaces. Network address: {}, port {}", local_ip()?, server_address.port()))?;

	poll.lock()?.registry().register(&mut listener, LISTENER, Interest::READABLE)?;

	// Start heartbeat
	{
		let heartbeat = Waker::new(poll.lock()?.registry(), HEARTBEAT)?;
		thread::spawn(move || { loop {
			thread::sleep(HEARTBEAT_TIME);
			heartbeat.wake().unwrap_or(()); // No shot this fails, but if it does, we don't care anyway.
		} });
	}
	/*// Start frame timer
	{
		let video_waker = Waker::new(poll.lock()?.registry(), VID_WAKER)?;
		thread::spawn(move || loop {
			thread::sleep(FRAME_TIME);
			video_waker.wake().unwrap_or(());
		});
	}*/


	// We will be implementing the TakeFlight server backend here. Since the process is spawned we can do our anything here
	let ownership_map = Arc::new(Mutex::new(HashMap::<Token, Connection>::new()));
	let mut event_buffer = Events::with_capacity(MAX_EVENTS);

	// test
	//let drone = crate::drone_interface::drone_pro::Drone::new(poll.clone(), ownership_map.clone(), logger.clone());
	let drone = crate::drone_interface::tello::drone::TelloDrone::new(poll.clone(), ownership_map.clone(), logger.clone())?;
	drone.lock()?.takeoff()?;
	sleep(Duration::from_secs(5));
	drone.lock()?.graceful_land()?;

	logger.info("Server starting!!!")?;

	let mut server = ServerInstance {
		listener,
		ownership_map,
		poll,
		logger 			: logger.clone(),
		video_src		: Arc::new(Mutex::new(None)),
		video_out		: Arc::new(Mutex::new(None)),
		drone_control	: None,
	};

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

								server.ownership_map.lock()?.insert(
									token,
									Connection::TCP(stream),
								);
							}
						Err(e) => {
							if e.kind() == ErrorKind::WouldBlock {continue}
							else { return Err(e.into()) }
						}
					}
				}
			}
			HEARTBEAT => {
				// Send heartbeat to all eligible connections
				server.logger.info("Sending out keep-alives!")?;
				let mut contacted_drones : Vec<Arc<Mutex<dyn Drone + 'static>>> = Vec::new();
				for connection in server.ownership_map.lock()?.iter_mut() {
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
			}
			VID_WAKER => {
				match server.send_image()
				{
					Err(Error::NoVideoSource) => { }
					Err(Error::NoVideoTarget) => { }
					Ok(_) => {  }
					e => { e? }
				}
			}
			token => {
				// FIXME: If we can do this without removing, that would be really cool.
				let found_connection = server.ownership_map.lock()?.remove(&token);
				match found_connection
				{
					Some(found) => {
						let connection = match found {
							Connection::Drone(drone) => {
								// Receive signal will always go until an error is encountered.
								// Below is the pattern matching for that error. We can recover from WouldBlock, but there are many layers of indirection.
								match drone.lock()?.receive_signal(token.0 as u16) {
									Err(e) => {
										match e
										{
											Error::IOError(io_error) => {
												if io_error.kind() == ErrorKind::WouldBlock { /* noop */ }
												else { Err(io_error)? }
											}
											_ => { Err(e)? }
										}
									}
									_ => { /* noop */ }
								}
								Connection::Drone(drone)
							}
							Connection::TCP(stream) => { handle_connection(stream, server)? }
							_ => { Err(Error::Custom("Error within drain_events token case. Did not know how to handle this connection..."))? }
						};

						server.ownership_map.lock()?.insert(token, connection);
					}
					// This seems like it should be an unrecoverable error, so I'm putting this here.
					None => { logger.error_from_string(format!("Unmapped port: {}", token.0))?; return Err(Error::Custom("Somehow registry included an unmapped value! Shutting down server!")) }
				}
			}
		}
	}

	Ok(())
}


impl ServerInstance
{
	fn send_image(&mut self) -> Result<(), Error>
	{
		// The only pessimization to this wrapper is the arc increment
		send_image(self.video_out.clone(), self.video_src.clone(), self.ownership_map.clone())
	}

}

pub(crate) fn send_image(
	out				: Arc<Mutex<Option<Token>>>,
	src				: Arc<Mutex<Option<Token>>>,
	ownership_map	: Arc<Mutex<HashMap<Token, Connection>>>,
) -> Result<(), Error>
{
	// While this is a large critical section, I actually think it's for the best, due to all the validations and possible reassignments of our streams.
	let mut video_out = out.lock()?;
	let mut video_src = src.lock()?;

	let video_out_token = video_out.ok_or(Error::NoVideoTarget)?;
	let video_src_token = video_src.ok_or(Error::NoVideoSource)?;

	let mut ownership_lock = ownership_map.lock()?;
	let src = ownership_lock.remove(&video_src_token).ok_or_else(|| {
		*video_src = None;
		Error::NoVideoSource
	})?;
	let mut out = ownership_lock.remove(&video_out_token).ok_or_else(|| {
		*video_out = None;
		Error::NoVideoTarget
	})?;

	let image = match &src {
		Connection::Drone(source) => {
			{
				let mut source_lock = source.lock()?;
				source_lock.snapshot().clone().ok_or(Error::Custom("Could not obtain image from drone!"))?.clone()
			}

		}
		Connection::Camera() => todo!(),
		_ => { Err(Error::NoVideoSource)? }
	};

	match out
	{
		Connection::VideoOut(cnx_type, ref mut stream) => {
			stream.write(&[u8::from(cnx_type.clone())])?;
			stream.write(&(image.len() as u16).to_be_bytes())?;
			stream.write_all(&image)?
		}
		_ => { Err(Error::NoVideoTarget)? }
	}

	ownership_lock.insert(video_src_token, src);
	ownership_lock.insert(video_out_token, out);

	Ok(())
}