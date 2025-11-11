use crate::drone_interface::{Drone, IUnit, Unit};
use crate::error::Error;
use crate::{drone_interface, Connection, Poll, Token, UdpSocket};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use crate::drone_interface::tello::packet::{land, set_sticks};
use mio::Interest;
use zerocopy::IntoBytes;
use crate::logger::Logger;
use super::packet;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TelloDrone
{
	command_sock	: UdpSocket,
	video_sock		: UdpSocket,
	info_sock		: UdpSocket,
	seq_number		: u16,
	response_buffer	: Vec<u8>,
	secret			: u16,		// This isn't actually encrypted, but its the value that the drone uses. Just for semantic use.

	forward_percent	: i16,
	sideway_percent	: i16,
	rotate_percent	: i16,
	updown_percent	: i16,

	inner_read_buf	: [u8;4096],
	logger			: Logger
}

impl drone_interface::Drone for TelloDrone
{
	fn takeoff(&mut self) -> Result<(), Error> {
		/*self.command_sock.send(b"takeoff")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));
*/
		self.command_sock.send(&packet::takeoff(self.seq_number))?;
		self.seq_number += 1;

		Ok(())
	}

	fn emergency_land(&mut self) -> Result<(), Error> {
		self.command_sock.send(b"emergency")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn graceful_land(&mut self) -> Result<(), Error> {
		/*self.command_sock.send(b"land")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())*/

		self.command_sock.send(&land(self.seq_number))?;
		self.seq_number += 1;
		Ok(())
	}

	fn up(&mut self, x: Unit) -> Result<(), Error> {
		self.command_sock.send(format!("up {x}").as_bytes())?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn down(&mut self, x: Unit) -> Result<(), Error> {
		self.command_sock.send(format!("down {x}").as_bytes())?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn forward(&mut self, x: Unit) -> Result<(), Error> {
		self.command_sock.send(format!("forward {x}").as_bytes())?;
		//self.command_sock.send(b"forward 25")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn backward(&mut self, x: Unit) -> Result<(), Error> {
		self.command_sock.send(format!("back {x}").as_bytes())?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn left(&mut self, x: Unit) -> Result<(), Error> {
		self.command_sock.send(format!("left {x}").as_bytes())?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn right(&mut self, x: Unit) -> Result<(), Error> {
		self.command_sock.send(format!("right {x}").as_bytes())?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn backflip(&mut self) -> Result<(), Error> {
		self.command_sock.send(b"flip b")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn frontflip(&mut self) -> Result<(), Error> {
		self.command_sock.send(b"flip f")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn clockwise_rot(&mut self, rads: f32) -> Result<(), Error> {
		self.command_sock.send(format!("cw {}", rads.to_degrees()).as_bytes())?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn cclockwise_rot(&mut self, rads: f32) -> Result<(), Error> {
		self.command_sock.send(format!("ccw {}", rads.to_degrees()).as_bytes())?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn snapshot(&mut self) -> Option<Arc<Vec<u8>>> {
		todo!()
	}

	fn rc(&mut self, lr: IUnit, ud: IUnit, fb: IUnit, rot: f32) -> Result<(), Error> {
		todo!()
	}

	fn send_heartbeat(&mut self) -> Result<(), Error> {
		self.command_sock.send(&set_sticks(self.seq_number, self.rotate_percent, self.updown_percent, self.sideway_percent, self.forward_percent))?;
		self.seq_number += 1;

		Ok(())
	}

	fn receive_signal(&mut self, port: u16) -> Result<(), Error> {
		if port == self.command_sock.local_addr()?.port()
		{
			loop {
				let bytes_read = self.command_sock.recv(&mut self.inner_read_buf)?;

				// Nab the last two bytes. In most messages this is the CRC or the secret.
				let packet_end = [self.inner_read_buf[bytes_read-2], self.inner_read_buf[bytes_read-1]];

				// This means that it's not formatted like the usual ones, and is probably plain text.
				if self.inner_read_buf[0] != 0xCC
				{
					let message = String::from_utf8_lossy(&self.inner_read_buf[..bytes_read]);
					self.logger.info_from_string(format!("[[Command Sock Message]]: {}" , crate::debug_utils::raw_hex_to_string(&self.inner_read_buf[..bytes_read])))?;

					if message.contains("conn_ack:")
					{
						if self.secret.as_bytes() != packet_end { Err(Error::Custom("Received the wrong value as acknowledgment from Tello."))? }
						else { self.logger.info("Received handshake acknowledgement from Tello!")? }
					}
					else if message.contains("unknown command: ")
					{
						self.logger.warn("We provided an unknown command to Tello.")?;
					}
					else {
						println!("no clue what's going on...");
						todo!("Non 0xCC packets from the Tello that weren't acknowledgement. The message: \"{}\"\t{}", message, message.len()) }

					// TODO: put something here.
				}
			}
		}
		else if port == self.video_sock.local_addr()?.port()
		{
			loop {
				let bytes_read = self.video_sock.recv(&mut self.inner_read_buf)?;
				self.logger.info_from_string(format!("[[Video Sock Message]]: {}" , crate::debug_utils::raw_hex_to_string(&self.inner_read_buf[..bytes_read])))?;
			}
		}
		else if port == self.info_sock.local_addr()?.port()
		{
			loop {
				let bytes_read = self.info_sock.recv(&mut self.inner_read_buf)?;
				self.logger.info_from_string(format!("[[Info Sock Message]]{}" , crate::debug_utils::raw_hex_to_string(&self.inner_read_buf[..bytes_read])))?;
			}
		}
		else { return Err(Error::Custom("Tello: Requested socket not found in this Tello!")) }
	}
}

impl TelloDrone
{
	#[allow(dead_code)]
	pub(crate) fn new(registry: Arc<Mutex<Poll>>, map: Arc<Mutex<HashMap<Token, Connection>>>, logger : Logger) -> Result<Arc<Mutex<Self>>, Error> {
		let mut command_sock = {
			const COMMAND_PORT: u16 = 8889;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, COMMAND_PORT);

			let command_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			command_sock.connect(SocketAddr::V4(SocketAddrV4::new(CONN_ADDR, COMMAND_PORT)))?;

			command_sock
		};

		let mut info_sock = {
			const INFO_PORT: u16 = 8890;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, INFO_PORT);
			let info_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			info_sock.connect(SocketAddr::V4(CONN_SOCK))?;

			info_sock
		};

		let mut video_sock = {
			const VIDEO_PORT: u16 = 11111;
			const ARBITRARY_PORT: u16 = 11112;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, VIDEO_PORT);

			let video_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			video_sock.connect(SocketAddr::V4(CONN_SOCK))?;

			video_sock
		};

		let com_token = Token(command_sock.local_addr()?.port() as usize);
		let vid_token = Token(video_sock.local_addr()?.port() as usize);
		let nfo_token = Token(info_sock.local_addr()?.port() as usize);

		logger.info_from_string(format!("Command socket connected to: {}",	com_token.0))?;
		logger.info_from_string(format!("Video socket connected to: {}",		vid_token.0))?;
		logger.info_from_string(format!("Info socket connected to: {}",		nfo_token.0))?;

		// Register all the sockets...
		{
			let poll_lock = registry.lock()?;
			let registry = poll_lock.registry();

			registry.register(&mut command_sock,	com_token, Interest::READABLE)?;
			registry.register(&mut video_sock,		vid_token, Interest::READABLE)?;
			registry.register(&mut info_sock,		nfo_token, Interest::READABLE)?;
		}

		let seq_number = 0;
		let response_buffer = vec![0; 255];

		sleep(Duration::from_secs(1));

		let this_drone = Arc::new(Mutex::new(Self {
			command_sock,
			video_sock,
			info_sock,
			seq_number,
			response_buffer,
			secret			: 0x1234_u16.to_be(), // Everything is little endian with the Tello, but this looks nice over the network.
			forward_percent	: 0,
			sideway_percent	: 0,
			rotate_percent	: 0,
			updown_percent	: 0,

			inner_read_buf	: [0;4096],
			logger,
		}
		));

		{
			let mut map_lock = map.lock()?;
			map_lock.insert(com_token, Connection::Drone(this_drone.clone()));
			map_lock.insert(vid_token, Connection::Drone(this_drone.clone()));
			map_lock.insert(nfo_token, Connection::Drone(this_drone.clone()));
		}

		this_drone.lock()?.connect()?;

		Ok(this_drone)
	}

	fn connect(&mut self) -> Result<(), Error>
	{
		let secret_bytes = self.secret.as_bytes();
		let conn_string = format!("conn_req:{}{}", secret_bytes[0] as char, secret_bytes[1] as char);
		self.command_sock.send(conn_string.as_bytes())?;

		Ok(())
	}
}