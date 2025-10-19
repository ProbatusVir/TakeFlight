use crate::drone_interface::Unit;
use crate::error::Error;
use crate::{drone_interface, Connection};
use mio::{Poll, Token};
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const WAIT_TIME : u64 = 3;

#[derive(Debug)]
pub struct Drone
{
	command_sock	: UdpSocket,
	video_sock		: UdpSocket,
	info_sock		: UdpSocket,
	#[allow(dead_code)]
	seq_number		: u16,			// Not in use right now.
	response_buffer	: Vec<u8>,
}

impl drone_interface::Drone for Drone
{
	fn takeoff(&mut self) -> Result<(), Error> {
		self.command_sock.send(b"takeoff")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn emergency_land(&mut self) -> Result<(), Error> {
		self.command_sock.send(b"emergency")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

		Ok(())
	}

	fn graceful_land(&mut self) -> Result<(), Error> {
		self.command_sock.send(b"land")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));

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

	fn snapshot(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn send_heartbeat(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn receive_signal(&mut self, port: u16) -> Result<(), Error> {
		todo!()
	}
}

impl Drone
{
	fn init(registry: Arc<Mutex<Poll>>, map: Arc<Mutex<HashMap<Token, Connection>>>, local_ip: IpAddr) -> Result<Arc<Mutex<Self>>, Error> {
		let command_sock = {
			const COMMAND_PORT: u16 = 8889;
			const ARBITRARY_PORT: u16 = 8889; // Operator
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, COMMAND_PORT);

			let command_sock = UdpSocket::bind(SocketAddr::new(local_ip, ARBITRARY_PORT))?;
			command_sock.connect(SocketAddrV4::new(CONN_ADDR, COMMAND_PORT))?;

			command_sock
		};

		let info_sock = {
			const INFO_PORT: u16 = 8890;
			const ARBITRARY_PORT: u16 = 8886;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, INFO_PORT);
			let info_sock = UdpSocket::bind(SocketAddr::new(local_ip, ARBITRARY_PORT))?;
			info_sock.connect(CONN_SOCK)?;

			info_sock
		};

		let video_sock = {
			const VIDEO_PORT: u16 = 11111;
			const ARBITRARY_PORT: u16 = 11112;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, VIDEO_PORT);

			let video_sock = UdpSocket::bind(SocketAddr::new(local_ip, ARBITRARY_PORT))?;
			video_sock.connect(CONN_SOCK)?;

			video_sock
		};

		let seq_number = 0;
		let response_buffer = vec![0; 255];

		command_sock.send(b"command")?;
		command_sock.send(b"streamon")?;

		sleep(Duration::from_secs(1));

		Ok(Arc::new(Mutex::new(Self { command_sock, video_sock, info_sock, seq_number, response_buffer })))
	}
}