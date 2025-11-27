use crate::computer_vision::HandLandmarker;
use crate::drone_interface::drone_pro::drone::DroneCommandState::BasicMovement;
use crate::drone_interface::{IUnit, Unit, _DroneInternal};
use crate::logger::Logger;
use crate::video::rtp;
use crate::video::rtp::{JpegMainHeader, RTPContent};
use crate::{Interest, ServerMap};
use crate::SocketAddr;
use crate::{drone_interface, app_network::send_image};
use crate::{Arc, Connection, Error, HashMap, Mutex, Poll, TcpStream, Token, UdpSocket};
use image::DynamicImage;
use image::ImageFormat::Jpeg;
use mio::event::Source;
use std::io::{Cursor, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::ops::BitXor;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Drone
{
	// I am assuming this is the handshake socket. I can't recall if I've seen other activity on this socket.
	handshake_sock			: UdpSocket,
	heartbeat_sock			: UdpSocket,
	video_sock				: UdpSocket,
	rtp_sock				: TcpStream,

	pub image				: Option<DynamicImage>,
	pub fin_image_buf		: Vec<u8>,
	last_frame_sent_time	: SystemTime,
	frame_time				: Arc<Duration>,
	inner_main_jpg_header	: Option<JpegMainHeader>,
	inner_raw_img_buf		: Vec<u8>,
	inner_read_buf			: [u8;4096],


	poll				: Arc<Mutex<Poll>>,
	connection_map		: Arc<Mutex<HashMap<Token, Connection>>>,
	curr_video_src		: Arc<Mutex<Option<Token>>>,
	curr_video_dst		: Arc<Mutex<Option<Token>>>,

	landmarker : HandLandmarker,
	logger : Logger
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum DroneCommandState
{
	BasicMovement	= 0,
	Takeoff			= 1,
	GracefulLand	= 2,
	EmergencyLand	= 4,
}

// It appears that the packet structure uses [0x03 0x66] as an indicator
//	that this is a command, and ends the transmission with [0x99]
// It also appears that all commands are 9 characters long.
/// We are allowing unused variable because development is not going ot be continued here.
#[allow(unused_variables)]

impl drone_interface::Drone for Drone
{

	fn takeoff(&mut self) -> Result<(), Error> {
		self.handshake_sock.send(&[0x3, 0x66, 0x80, 0x80, 0x80, 0x80, 0x0, 0x0, 0x99])?;
		self.handshake_sock.send(&[0x3, 0x66, 0x80, 0x80, 0x80, 0x80, 0x01, 0x01, 0x99])?;

		Ok(())
	}

	fn emergency_land(&mut self) -> Result<(), Error> {
		self.handshake_sock.send(&[0x3, 0x66, 0x80, 0x80, 0x80, 0x80, 0x0, 0x0, 0x99])?;
		self.handshake_sock.send(&[0x3, 0x66, 0x80, 0x80, 0x80, 0x80, 0x4, 0x4, 0x99])?;

		Ok(())
	}

	fn graceful_land(&mut self) -> Result<(), Error> {
		self.handshake_sock.send(&[0x3, 0x66, 0x80, 0x80, 0x80, 0x80, 0x0, 0x0, 0x99])?;
		self.handshake_sock.send(&[0x3, 0x66, 0x80, 0x80, 0x80, 0x80, 0x02, 0x02, 0x99])?;

		Ok(())
	}

	fn up(&mut self, x: Unit) -> Result<(), Error> {
		self.create_command(0, 0, -0x6b, 0, BasicMovement)

	}

	fn down(&mut self, x: Unit) -> Result<(), Error> {
		self.create_command(0, 0, 0x6a, 0, BasicMovement)
	}

	fn forward(&mut self, x: Unit) -> Result<(), Error> {
		self.create_command(0, -0x02, 0, 0, BasicMovement)
	}

	fn backward(&mut self, x: Unit) -> Result<(), Error> {
		self.create_command(0, 0x1e, 0, 0, BasicMovement)

	}

	fn left(&mut self, x: Unit) -> Result<(), Error> {
		self.create_command(-0x15, 0, 0, 0, BasicMovement)

	}

	fn right(&mut self, x: Unit) -> Result<(), Error> {
		self.create_command(0x25, 0, 0, 0, BasicMovement)
	}

	fn backflip(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn frontflip(&mut self) -> Result<(), Error> {
		todo!()
	}

	fn clockwise_rot(&mut self, rads: f32) -> Result<(), Error> {
		self.create_command(0, 0, 0, 0x58, BasicMovement)
	}

	fn cclockwise_rot(&mut self, rads: f32) -> Result<(), Error> {
		self.create_command(0, 0, 0,-0x79, BasicMovement)
	}

	fn snapshot(&mut self) -> Option<Arc<DynamicImage>> {
		Some(Arc::new(self.image.clone()?))
	}

	fn rc(&mut self, lr: IUnit, ud: IUnit, fb: IUnit, rot: f32) -> Result<(), Error> {
		// FIXME: rot *should* always be 0 right now. We obviously values from -128 to 127
		self.create_command(lr as i8, ud as i8, fb as i8, rot as i8, BasicMovement)
	}

	fn send_heartbeat(&mut self) -> Result<(), Error> {
		self.heartbeat_sock.send(&[0xef, 0x00, 0x04, 0x00])?;
		self.handshake_sock.send(&[0x01, 0x01])?;

		Ok(())
	}

	fn receive_signal(&mut self, port: u16) -> Result<(), Error> {
		if port == self.video_sock.local_addr()?.port() {
			self.logger.info("Drone Pro is receiving video packets")?;
			loop
			{
				let bytes_read = self.video_sock.recv(&mut self.inner_read_buf)?;
				let mut cursor = Cursor::new(&self.inner_read_buf);
				// Strip the RTP header from the stream
				let new_header = rtp::RTPHeader::from_stream(&mut cursor)?;

				// we're gonna assume that the images are all sent in order.
				{
					// Add only the jpeg data. Markers and payload
					let cursor_position = cursor.position() as usize;
					// FIXME: this is debug only!
					debug_assert!(cursor_position <= bytes_read);
					self.inner_raw_img_buf.extend_from_slice(&self.inner_read_buf[cursor_position..bytes_read]);

				}

				if new_header.content_header.is_some()
				{
					match new_header.content_header.unwrap() {
						RTPContent::Jpeg(jpeg_header) => {
							if jpeg_header.is_image_start()
							{
								self.inner_main_jpg_header = Some(jpeg_header)
							}
						}
						#[allow(unreachable_patterns)]
						_ => { Err(Error::RTPTypeNotImplemented(new_header.payload_type))? }
					}
				}

				// I suppose it's possible for a packet to be both start and finish.
				if new_header.is_last_in_frame {
					let mut lqt : [u8;64] = [0;64];
					let mut cqt : [u8;64] = [0;64];

					// This avoids a fatal error where this value is unwrapped below.
					// This covers the case where we do not receive the first packet of the image.
					if self.inner_main_jpg_header.is_none()
					{
						self.cleanup_image();
						continue;
					}
					let main_jpeg_header = self.inner_main_jpg_header.as_ref().unwrap();
					let quant_header = main_jpeg_header.quantization_header.as_ref().unwrap();

					lqt.clone_from_slice(&quant_header.table[..64]);
					cqt.clone_from_slice(&quant_header.table[64..]);

					crate::video::rfc2435::create_image(&mut self.fin_image_buf,
														&main_jpeg_header, &mut self.inner_raw_img_buf,
														&mut lqt, &mut cqt, None)?;

					let image_result = image::load_from_memory_with_format(&self.fin_image_buf, Jpeg);
					match image_result
					{
						Ok(img) => {
							self.image = Some(img);
						}
						Err(_) => { self.image = None; }
					};

					// Send the image to the client, if possible.
					let now = SystemTime::now();
					if now.duration_since(self.last_frame_sent_time)? >= *self.frame_time
					{
						todo!("Why are you using the DronePro?")
						/*match send_image(self.curr_video_dst.clone(), self.curr_video_src.clone(), self.connection_map.clone())
						{
							Err(Error::NoVideoSource) => { }
							Err(Error::NoVideoTarget) => { }
							Ok(_) => {  }
							e => { e? }
						}*/
					}

					self.last_frame_sent_time = now;

					/* TODO: image processing code goes here. */

					self.cleanup_image();
				}

			}

		}
		else if port == self.rtp_sock.local_addr()?.port() {
			self.logger.info("Received out-of-band video information from RTP protocol.")?;
			let bytes_read = self.rtp_sock.read(&mut self.inner_read_buf)?;
			dbg!("RPT SOCKET: {}", &[..bytes_read]);
			Ok(())
		}
		else if port == self.heartbeat_sock.local_addr()?.port() { self.logger.warn("Received unexpected packet from handshake socket...")?; Ok(()) }
		else if port == self.handshake_sock.local_addr()?.port() {
			self.logger.warn("Received unexpected packet from handshake socket...")?;

			Ok(())
		}
		else { return Err(Error::Custom("DronePro: Requested socket not found in DronePro!")) }

	}
}

impl Drop for Drone
{
	fn drop(&mut self) {
		// If we cannot lock the poll to unregister these sockets, this is an unrecoverable error
		{
			let poll_lock = self.poll.lock().unwrap();
			let registry = poll_lock.registry();
			self.handshake_sock.deregister(registry).unwrap();
			self.heartbeat_sock.deregister(registry).unwrap();
			self.video_sock.deregister(registry).unwrap();
			self.rtp_sock.deregister(registry).unwrap()
		}

		{
			let mut map_lock = self.connection_map.lock().unwrap();
			map_lock.remove(&Token(self.handshake_sock.local_addr().unwrap().port() as usize));
			map_lock.remove(&Token(self.heartbeat_sock.local_addr().unwrap().port() as usize));
			map_lock.remove(&Token(self.video_sock.local_addr().unwrap().port() as usize));
			map_lock.remove(&Token(self.rtp_sock.local_addr().unwrap().port() as usize));
		}
	}
}


impl Drone
{
	pub(crate) fn new(poll: Arc<Mutex<Poll>>, connection_map: Arc<Mutex<HashMap<Token, Connection>>>, logger : Logger,
					  curr_video_src : Arc<Mutex<Option<Token>>>, curr_video_dst : Arc<Mutex<Option<Token>>>,  frame_time : Arc<Duration>, /*hand_landmarker : Arc<Mutex<HandLandmarker>>*/) -> Result<Arc<Mutex<Self>>, Error>
	where
		Self: Sized
	{

		let video_sock = {
			let poll_lock = poll.lock()?;
			let registry = poll_lock.registry();
			let mut video_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 30732)))?;
			let port = video_sock.local_addr()?.port() as usize;
			registry.register(&mut video_sock, Token(port), Interest::READABLE)?;

			video_sock
		};

		let mut handshake_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
		handshake_sock.connect("192.168.1.1:7099".parse()?)?;
		handshake_sock.send(&[0x01, 0x01])?;

		let mut heartbeat_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
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
		{
			let video_start = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?; // this number matters since the drone initiates
			video_start.connect("192.168.1.1:52612".parse()?)?;
			video_start.send(&[0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0])?;
		}
		let inner_read_buf = [0_u8;4096];



		let handshake_token	= Token(handshake_sock.local_addr()?.port() as usize);
		let heartbeat_token	= Token(heartbeat_sock.local_addr()?.port() as usize);
		let rtp_token		= Token(rtp_sock.local_addr()?.port() as usize);
		let video_token		= Token(video_sock.local_addr()?.port() as usize);

		// register all sockets to poll
		{
			let poll_lock = poll.lock()?;
			poll_lock.registry().register(&mut handshake_sock,	handshake_token,	Interest::READABLE)?;
			poll_lock.registry().register(&mut heartbeat_sock,	heartbeat_token,	Interest::READABLE)?;
			poll_lock.registry().register(&mut rtp_sock,		rtp_token,			Interest::READABLE)?;

			// If you're wondering why this is commented out, look at the top of the function.
			// This is still here to make it apparent that this is not an oversight.
			// -- original line written 10/18/25. -- this line was written on 11/9/25.
			//poll_lock.registry().register(&mut video_sock,		video_token,		Interest::READABLE)?;
		}

		let this_drone = Arc::new(Mutex::new(Self {
			handshake_sock,
			heartbeat_sock,
			video_sock,
			rtp_sock,
			inner_raw_img_buf: Default::default(),
			fin_image_buf	: Default::default(),
			inner_read_buf,
			poll			: poll.clone(),
			connection_map	: connection_map.clone(),
			curr_video_src,
			inner_main_jpg_header: None,
			image			: None,
			landmarker: HandLandmarker::from_path("src/model/hand_landmarks_detector.tflite")?,
			logger,
			last_frame_sent_time : UNIX_EPOCH,
			curr_video_dst,
			frame_time,
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

	fn cleanup_image(&mut self)
	{
		self.inner_raw_img_buf.clear();
		self.fin_image_buf.clear();
		self.inner_main_jpg_header = None;
	}

	fn spin_rotors(&mut self, unkn1 : u8) -> Result<(), Error>
	{
		// Looks like the number xor the last bit...
		let unkn2 = unkn1.bitxor(0b1000_0000);
		self.handshake_sock.send(&[0x03, 0x66, 0x80, 0x80, unkn1, 0x80, 0x00, unkn2, 0x099])?;

		Ok(())
	}

	/// lr: left/right
	///
	/// fb: front/back
	///
	/// ud: up/down
	///
	/// r: rotate
	fn create_command(&mut self, lr : i8, fb : i8, ud : i8, r : i8, cmd : DroneCommandState ) -> Result<(), Error>
	{
		const DEFAULT : i8 = 0x80u8 as i8;
		let checksum = lr ^ fb ^ ud ^ r ^ (cmd as i8);

		self.handshake_sock.send(
			&[0x03, 0x66,
			DEFAULT.wrapping_add(lr) as u8,
			DEFAULT.wrapping_add(fb) as u8,
			DEFAULT.wrapping_add(ud) as u8,
			DEFAULT.wrapping_add(r ) as u8,
			cmd as u8,
			checksum as u8,
			0x99])?;

		Ok(())
	}
}

impl _DroneInternal for Drone
{
	fn expose_video_stream_port(&self) -> Result<u16, Error> {
		Ok(self.video_sock.local_addr()?.port())
	}

	fn expose_video_stream(&mut self) -> &mut UdpSocket {
		&mut self.video_sock
	}

	fn expose_ownership_map(&self) -> ServerMap {
		self.connection_map.clone()
	}

	fn expose_server_src_token(&self) -> Arc<Mutex<Option<Token>>> {
		self.curr_video_src.clone()
	}

	fn expose_server_out_token(&self) -> Arc<Mutex<Option<Token>>> {
		self.curr_video_dst.clone()
	}
}