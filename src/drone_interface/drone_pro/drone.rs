use crate::Interest;
use crate::SocketAddr;
use std::net::IpAddr;
use crate::drone_interface;
use crate::drone_interface::Unit;
use crate::{Error, Arc, Mutex, HashMap, Connection, Token, Poll, UdpSocket, TcpStream};
use std::io::{Write, Read};
use std::str::FromStr;
pub struct Drone
{
	video_frame		: usize,
	// I am assuming this is the handshake socket. I can't recall if I've seen other activity on this socket.
	handshake_sock	: UdpSocket,
	heartbeat_sock	: UdpSocket,
	video_sock		: UdpSocket,
	rtp_sock		: TcpStream,
	frame_buffer	: [u8;4096],
	rtp_count		: u8,			// This is janky, and I don't like it.
	poll			: Arc<Mutex<Poll>>,
	connection_map	: Arc<Mutex<HashMap<Token, Connection>>>,
}

impl drone_interface::Drone for Drone
{
	fn init(poll: Arc<Mutex<Poll>>, connection_map: Arc<Mutex<HashMap<Token, Connection>>>, local_ip: IpAddr) -> Result<Arc<Mutex<Self>>, Error>
	where
		Self: Sized
	{
		let mut handshake_sock = UdpSocket::bind(SocketAddr::new(local_ip, 0))?;
		handshake_sock.connect("192.168.1.1:7099".parse()?)?;
		handshake_sock.send(&[0x01, 0x01])?;

		let mut heartbeat_sock = UdpSocket::bind(SocketAddr::new(local_ip, 0))?;
		heartbeat_sock.connect("192.168.169.1:8800".parse()?)?;

		// TODO: Make this properly non-blocking
		let mut rtp_sock = std::net::TcpStream::connect(SocketAddr::new("192.168.1.1".parse()?, 7070))?;
		rtp_sock.write(b"OPTIONS rtsp://192.168.1.1:7070/webcam RTSP/1.0\x0d\x0aCSeq: 1\x0d\x0aUser-Agent: Lavf57.71.100\x0d\x0a\x0d\x0a")?;

		// TODO: This block can be gotten rid of once the above is done.
		{
			let mut tcp_input_buffer = vec![0;256];
			rtp_sock.read(&mut tcp_input_buffer)?;
			//println!("{}", String::from_utf8_lossy(&tcp_input_buffer));

			// Write packet 2
			rtp_sock.write(&[0x44, 0x45, 0x53, 0x43, 0x52, 0x49, 0x42, 0x45, 0x20, 0x72, 0x74, 0x73, 0x70, 0x3a, 0x2f, 0x2f, 0x31, 0x39, 0x32, 0x2e, 0x31, 0x36, 0x38, 0x2e, 0x31, 0x2e, 0x31, 0x3a, 0x37, 0x30, 0x37, 0x30, 0x2f, 0x77, 0x65, 0x62, 0x63, 0x61, 0x6d, 0x20, 0x52, 0x54, 0x53, 0x50, 0x2f, 0x31, 0x2e, 0x30, 0xd, 0xa, 0x41, 0x63, 0x63, 0x65, 0x70, 0x74, 0x3a, 0x20, 0x61, 0x70, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x2f, 0x73, 0x64, 0x70, 0xd, 0xa, 0x43, 0x53, 0x65, 0x71, 0x3a, 0x20, 0x32, 0xd, 0xa, 0x55, 0x73, 0x65, 0x72, 0x2d, 0x41, 0x67, 0x65, 0x6e, 0x74, 0x3a, 0x20, 0x4c, 0x61, 0x76, 0x66, 0x35, 0x37, 0x2e, 0x37, 0x31, 0x2e, 0x31, 0x30, 0x30, 0xd, 0xa, 0xd, 0xa])?;
			rtp_sock.read(&mut tcp_input_buffer)?;
			println!("{}", String::from_utf8_lossy(&tcp_input_buffer));



			// Write packet 3
			rtp_sock.write(&[0x53, 0x45, 0x54, 0x55, 0x50, 0x20, 0x72, 0x74, 0x73, 0x70, 0x3a, 0x2f, 0x2f, 0x31, 0x39, 0x32, 0x2e, 0x31, 0x36, 0x38, 0x2e, 0x31, 0x2e, 0x31, 0x3a, 0x37, 0x30, 0x37, 0x30, 0x2f, 0x77, 0x65, 0x62, 0x63, 0x61, 0x6d, 0x2f, 0x74, 0x72, 0x61, 0x63, 0x6b, 0x30, 0x20, 0x52, 0x54, 0x53, 0x50, 0x2f, 0x31, 0x2e, 0x30, 0xd, 0xa, 0x54, 0x72, 0x61, 0x6e, 0x73, 0x70, 0x6f, 0x72, 0x74, 0x3a, 0x20, 0x52, 0x54, 0x50, 0x2f, 0x41, 0x56, 0x50, 0x2f, 0x55, 0x44, 0x50, 0x3b, 0x75, 0x6e, 0x69, 0x63, 0x61, 0x73, 0x74, 0x3b, 0x63, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x5f, 0x70, 0x6f, 0x72, 0x74, 0x3d, 0x33, 0x30, 0x37, 0x33, 0x32, 0x2d, 0x33, 0x30, 0x37, 0x33, 0x33, 0xd, 0xa, 0x43, 0x53, 0x65, 0x71, 0x3a, 0x20, 0x33, 0xd, 0xa, 0x55, 0x73, 0x65, 0x72, 0x2d, 0x41, 0x67, 0x65, 0x6e, 0x74, 0x3a, 0x20, 0x4c, 0x61, 0x76, 0x66, 0x35, 0x37, 0x2e, 0x37, 0x31, 0x2e, 0x31, 0x30, 0x30, 0xd, 0xa, 0xd, 0xa])?;
			rtp_sock.read(&mut tcp_input_buffer)?;

			let session_id = {
				let session_description = String::from_utf8_lossy(&tcp_input_buffer);
				let mut session_id = String::new();
				for line in session_description.lines()
				{
					let result = line.split_once("Session: ");
					if result.is_some() { session_id = String::from_str(result.unwrap().1)? }
				}
				session_id
			};

			// \r\n

			// Write packet 4
			rtp_sock.write(format!("PLAY rtsp://192.168.1.1:7070/webcam/ RTSP/1.0\r\nRange: npt=0.000-\r\nCSeq: 4\r\nUser-Agent: Lavf57.71.100\r\nSession: {session_id}\r\n\r\n")
				.as_bytes())?;
			rtp_sock.read(&mut tcp_input_buffer)?;
		}

		let mut rtp_sock = mio::net::TcpStream::from_std(rtp_sock);

		/* Just for one send, it looks like. */
		let video_sock = UdpSocket::bind(SocketAddr::new(local_ip, 0))?; // this number matters since the drone initiates
		video_sock.connect("192.168.1.1:52612".parse()?)?;
		video_sock.send(&[0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0])?;

		let mut video_sock = UdpSocket::bind(SocketAddr::new(local_ip, 30732))?;

		let handshake_token	= Token(handshake_sock.local_addr()?.port() as usize);
		let heartbeat_token	= Token(heartbeat_sock.local_addr()?.port() as usize);
		let rtp_token		= Token(rtp_sock.local_addr()?.port() as usize);
		let video_token		= Token(video_sock.local_addr()?.port() as usize);

		// register all sockets to poll
		{
			let mut poll_lock = poll.lock()?;
			poll_lock.registry().register(&mut handshake_sock,	handshake_token,	Interest::READABLE)?;
			poll_lock.registry().register(&mut heartbeat_sock,	heartbeat_token,	Interest::READABLE)?;
			poll_lock.registry().register(&mut rtp_sock,		rtp_token,			Interest::READABLE)?;
			poll_lock.registry().register(&mut video_sock,		video_token,		Interest::READABLE)?;
		}

		let this_drone = Arc::new(Mutex::new(Self {
			video_frame		: 0,
			handshake_sock,
			heartbeat_sock,
			video_sock,
			rtp_sock,
			frame_buffer	: [0;4096],
			rtp_count		: 0,			// This is janky, and I don't like it.
			poll			: poll.clone(),
			connection_map : connection_map.clone(),
		}));

		// Register all sockets to map
		{
			let mut map_lock = connection_map.lock()?;
			map_lock.insert(handshake_token,	Connection::Drone(this_drone.clone()));
			map_lock.insert(heartbeat_token,	Connection::Drone(this_drone.clone()));
			map_lock.insert(rtp_token,			Connection::Drone(this_drone.clone()));
			map_lock.insert(video_token,		Connection::Drone(this_drone.clone()));
		}

		Ok(this_drone)
	}

	fn takeoff(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn emergency_land(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn graceful_land(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn up(&mut self, x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn down(&mut self, x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn forward(&mut self, x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn backward(&mut self, x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn left(&mut self, x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn right(&mut self, x: Unit) -> Result<(), Error> {
		todo!()
	}

	fn backflip(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn frontflip(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn clockwise_rot(&mut self, rads: f32) -> Result<(), Error> {
		todo!()
	}

	fn cclockwise_rot(&mut self, rads: f32) -> Result<(), Error> {
		todo!()
	}

	fn snapshot(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn send_heartbeat(&mut self) -> Result<(), Error> {
		self.heartbeat_sock.send(&[0xef, 0x00, 0x04, 0x00])?;

		Ok(())
	}

	fn receive_signal(&mut self, port: u16) -> Result<(), Error> {
		if port == self.video_sock.local_addr()?.port() {
			loop
			{
				self.video_sock.recv(&mut self.frame_buffer);
				dbg!("Received video thing");
			}
			Ok(())
		}
		else if port == self.rtp_sock.local_addr()?.port() { todo!() }
		else if port == self.heartbeat_sock.local_addr()?.port() { todo!() }
		else if port == self.handshake_sock.local_addr()?.port() { self.handshake_sock.recv(&mut self.frame_buffer); dbg!("Received a response from handshake socket"); Ok(())/*todo!("DronePro: Handshake socket sent a new packet.")*/ }
		else { return Err(Error::Custom("DronePro: Requested socket not found in DronePro!")) }

	}
}

//FIXME: we need a drop method here!!!