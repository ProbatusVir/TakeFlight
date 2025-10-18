mod helper;
mod video_stream;
mod drone_interface;
mod error;
#[cfg(test)]
mod tests;
#[cfg(debug_assertions)]
pub(crate) mod debug_utils;

mod computer_vision;
mod video;
mod http_server;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tflitec as tf;

use error::Error;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use mio;
use mio::{Events, Interest, Poll, Registry, Token, Waker};
use mio::net::{TcpStream, UdpSocket, TcpListener};
use crate::drone_interface::Drone;

#[derive(Debug)]
enum Connection
{
	TCP(TcpStream),
	UDP(UdpSocket),
	Drone(Arc<Mutex<dyn Drone>>)
}

const LISTENER : Token = Token(0);	// 0 is the reserved file descriptor for stdin. It cannot be used for ports, so listener is always valid.
const HEARTBEAT : Token = Token(1); // 1 is reserved by the system for stdout. (2 is stdout, we can use it as well.)
//Main fn that executes the application within a localhost http with the return signature Result<(), Error>
//Allowing for proper error handling in case the application can not be opened
fn main() -> Result<(), Error> {
	const MAX_EVENTS : usize = 1024;
	let heartbeat_time: Duration = Duration::from_secs_f32(3.0);

	println!("Hello, world!");

	// Start the server
	let server_address = local_ip_address::local_ip()?;
	let mut poll = Arc::new(Mutex::new(Poll::new()?));
	let mut listener = mio::net::TcpListener::bind(SocketAddr::new(server_address, 0))?;


	// TODO: The application should not try to be the server. This will be cleaner when we sort this out.
	//let port = listener.local_addr()?.port();
	let port = 5173;
	poll.lock()?.registry().register(&mut listener, LISTENER, Interest::READABLE)?;

	// Start the application
	let mut application_status = Command::new("cmd")
		.args(["/C",
			&format!("start http://localhost:{port}")])
		.spawn()?;

	// Start heartbeat
	{
		let mut heartbeat = Waker::new(poll.lock()?.registry(), HEARTBEAT)?;
		thread::spawn(move || { loop {
			thread::sleep(heartbeat_time);
			heartbeat.wake().unwrap_or(()); // No shot this fails, but if it does, we don't care anyway.
		} });
	}


	// We will be implementing the TakeFlight server backend here. Since the process is spawned we can do our anything here
	let mut ownership_map = Arc::new(Mutex::new(HashMap::<Token, Connection>::new()));
	let mut event_buffer = Events::with_capacity(MAX_EVENTS);

	// test
	//let drone = crate::drone_interface::drone_pro::drone::Drone::init(poll.clone(), ownership_map.clone(), server_address);

	// Some multiplexing
	let status = loop
	{
		// Receive and handle events
		poll.lock()?.poll(&mut event_buffer, None)?;
		let events_result = drain_events(&mut event_buffer, &mut listener, &mut ownership_map, &mut poll);

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

	println!("React application exited with status: {}", application_status.wait()?);

	status
}

fn drain_events(event_buffer	: &mut Events,
				listener		: &mut TcpListener,
				ownership_map	: &mut Arc<Mutex<HashMap<Token, Connection>>>,
				registry 		: &mut Arc<Mutex<Poll>>)
	-> Result<(), Error>
{
	for event in event_buffer.iter()
	{
		match event.token()
		{
			LISTENER => {
				// Accept all incoming streams.
				loop {
					let incoming = listener.accept();
					match incoming
					{
						Ok((mut stream, address)) => {
							let token = Token(address.port() as usize);
								registry.lock()?.registry().register(
									&mut stream,
									token.clone(),
									Interest::READABLE)?;

								ownership_map.lock()?.insert(
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
				for mut connection in ownership_map.lock()?.iter_mut() {
					match connection.1
					{
						// TODO: This is sorely in need of a refactor...
						Connection::Drone(drone) => {
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
				match ownership_map.lock()?.get_mut(&token)
				{
					Some(found) => {
						match found {
							Connection::Drone(drone) => { drone.lock()?.receive_signal(token.0 as u16)?; }
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