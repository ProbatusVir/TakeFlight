mod helper;
mod video_stream;
mod drone_interface;
mod error;
#[cfg(test)]
mod tests;
#[cfg(debug_assertions)]
pub(crate) mod debug_utils;

mod video;
pub(crate) mod logger;
mod app_network;

use crate::drone_interface::Drone;
use error::Error;
use mio;
use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Events, Interest, Poll, Token, Waker};
use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::io::ErrorKind::ConnectionAborted;
use std::net::SocketAddr;
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::logger::{do_logging, Logger};
use httparse::Status;
use serde::{Deserialize, Serialize};
use takeflight_computer_vision as computer_vision;
use crate::app_network::{handle_connection, ClientSocketType};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Connection
{
	TCP(TcpStream),
	UDP(UdpSocket),
	Client(ClientSocketType, TcpStream),
	VideoOut(ClientSocketType, TcpStream), // This one is for sending video to the client. There will be a "VideoIn," which will be used for the CV pipeline.
	Drone(Arc<Mutex<dyn Drone>>),
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

#[derive(Serialize, Deserialize)]
struct DroneNames{
	names: Vec<String>
}
const LISTENER : Token = Token(0);	// 0 is the reserved file descriptor for stdin. It cannot be used for ports, so listener is always valid.
const HEARTBEAT : Token = Token(1); // 1 is reserved by the system for stdout. (2 is stdout, we can use it as well.)
const LOG_DIR : &str = "logs/";

// TODO: Let's make a more accurate name for this.
struct ServerInstance
{
	// video_stream : //TODO: let's make an enum where it can be a drone, or a separate TCP
	event_buffer	: Events,
	listener		: TcpListener,
	ownership_map	: Arc<Mutex<HashMap<Token, Connection>>>,
	poll 			: Arc<Mutex<Poll>>,
	logger			: Logger
}

//Main fn that executes the application within a localhost http with the return signature Result<(), Error>
//Allowing for proper error handling in case the application can not be opened
fn main() -> Result<(), Error> {
	const MAX_EVENTS : usize = 1024;
	let heartbeat_time: Duration = Duration::from_secs_f32(3.0);

	println!("Hello, world!");

	// TODO: Add logic for determining log file.
	let log_file = "log_file";
	std::fs::create_dir(LOG_DIR).unwrap_or_default(); // Make sure the file directory exists
	let file = Arc::new(Mutex::new(Some(File::create(format!("{LOG_DIR}{log_file}"))?)));
	let (logger, receiver) = logger::Logger::new();

	// Start the logger
	{
		let cloned_file = file.clone();
		thread::spawn(move | | { do_logging(receiver, file).unwrap() });
	}

	logger.info(String::from_str("Logger started!")?)?;

	// Start the server
	let server_address = local_ip_address::local_ip()?;
	let poll = Arc::new(Mutex::new(Poll::new()?));
	let mut listener = TcpListener::bind(SocketAddr::new(server_address, 0))?;

	poll.lock()?.registry().register(&mut listener, LISTENER, Interest::READABLE)?;

	// Start heartbeat
	{
		let heartbeat = Waker::new(poll.lock()?.registry(), HEARTBEAT)?;
		thread::spawn(move || { loop {
			thread::sleep(heartbeat_time);
			heartbeat.wake().unwrap_or(()); // No shot this fails, but if it does, we don't care anyway.
		} });
	}


	// We will be implementing the TakeFlight server backend here. Since the process is spawned we can do our anything here
	let ownership_map = Arc::new(Mutex::new(HashMap::<Token, Connection>::new()));
	let event_buffer = Events::with_capacity(MAX_EVENTS);

	// test
	//let drone = crate::drone_interface::drone_pro::drone::Drone::new(poll.clone(), ownership_map.clone(), server_address);

	logger.info(String::from_str("Server starting!!!")?)?;

	let mut server = ServerInstance {
		event_buffer,
		listener,
		ownership_map,
		poll,
		logger,
	};

	// Some multiplexing
	let status = loop
	{
		// Receive and handle events
		server.poll.lock()?.poll(&mut server.event_buffer, None)?;
		let events_result = drain_events(&mut server);

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
fn drain_events(server: &mut ServerInstance)
				-> Result<(), Error>
{
	for event in server.event_buffer.iter()
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
				server.logger.info(String::from_str("Sending out keep-alives!")?)?;
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
			token => {
				let mut ownership_map_lock = server.ownership_map.lock()?;
				match ownership_map_lock.get_mut(&token)
				{
					Some(found) => {
						match found {
							Connection::Drone(drone) => { drone.lock()?.receive_signal(token.0 as u16)?; }
							Connection::TCP(stream) => {
								let stream = ownership_map_lock.remove(&token).unwrap();
								handle_connection(stream.try_into()?, &mut *ownership_map_lock)?; }
							_ => { /* noop until we get the application connected */ }
						}
					}
					// This seems like it should be an unrecoverable error, so I'm putting this here.
					None => { return Err(Error::Custom("Somehow registry included an unmapped value! Shutting down server!")) }
				}
			}
		}
	}

	Ok(())
}
