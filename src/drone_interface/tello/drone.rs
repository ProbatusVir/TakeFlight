use std::net::{ Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use local_ip_address::local_ip;
use crate::drone_interface;
use crate::drone_interface::Unit;
use crate::error::Error;



struct Drone
{
	command_sock	: UdpSocket,
	video_sock		: UdpSocket,
	info_sock		: UdpSocket,
	seq_number		: u16,
}

impl drone_interface::Drone for Drone
{
	fn init() -> Result<Self, Error> {
		let local_ip = local_ip()?;
		let command_sock = {
				const COMMAND_PORT : u16 = 8889;
				const ARBITRARY_PORT : u16 = 25565;
				const CONN_ADDR : Ipv4Addr = Ipv4Addr::new(192, 168, 0, 1);
				let mut command_sock = UdpSocket::bind(SocketAddr::new(local_ip, ARBITRARY_PORT))?;
				command_sock.connect(SocketAddrV4::new(CONN_ADDR, COMMAND_PORT) )?;

				command_sock
			};

		let video_sock = todo!();
		let info_sock = todo!();
		let seq_number = 0;

		Ok( Self {command_sock, video_sock, info_sock, seq_number })
	}

	fn takeoff() -> Result<(), Error> {
		todo!()
	}

	fn emergency_land() -> Result<(), Error> {
		todo!()
	}

	fn graceful_land() -> Result<(), Error> {
		todo!()
	}

	fn forward(x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn backward(x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn left(x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn right(x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn clockwise_rot(rads: f32) -> Result<(), Error> {
		todo!()
	}

	fn cclockwise_rot(rads: f32) -> Result<(), Error> {
		todo!()
	}
}