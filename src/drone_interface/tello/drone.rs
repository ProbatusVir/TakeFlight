use super::packet;
use crate::app_network::DroneStateJSON;
use crate::drone_interface::tello::packet::{land, set_sticks, strip_payload, Command, FlightData};
use crate::drone_interface::{IUnit, Unit, _DroneInternal};
use crate::error::Error;
use crate::logger::Logger;
use crate::video::video_queue::{FrameType, VideoQueue};
use crate::{debug_utils, drone_interface, Connection, Poll, ServerMap, Token, UdpSocket};
use concat_arrays::concat_arrays;
use image::DynamicImage;
use mio::event::Source;
use mio::Interest;
use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zerocopy::IntoBytes;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TelloDrone
{
	command_sock	: UdpSocket,
	video_sock		: UdpSocket,
	is_connected	: bool,
	time_created	: SystemTime,

	seq_number		: u16,
	response_buffer	: Vec<u8>,
	vid_frame_number: u8,
	frame_buffer	: Vec<u8>,
	image			: Option<DynamicImage>,

	last_sps_pps_req: SystemTime,
	sps				: Option<[u8;15 - 2]>,
	pps				: Option<[u8;10 - 2]>,
	idr				: Vec<u8>,
	idr_frame_number: Option<u8>,

	forward_percent	: i16,
	sideway_percent	: i16,
	rotate_percent	: i16,
	updown_percent	: i16,
	battery_percent	: u8,

	inner_read_buf	: [u8;4096],
	logger			: Logger,

	poll					: Arc<Mutex<Poll>>,
	connection_map			: Arc<Mutex<HashMap<Token, Connection>>>,
	frame_time				: Arc<Duration>,
	last_frame_sent_time	: SystemTime,
	curr_video_src			: Arc<Mutex<Option<Token>>>,
	curr_video_dst			: Arc<Mutex<Option<Token>>>,

	curr_state				: Option<FlightData>,

	video_queue 			: VideoQueue,

}

impl drone_interface::Drone for TelloDrone
{
	fn takeoff(&mut self) -> Result<(), Error> {
		/*self.command_sock.send(b"takeoff")?;
		self.command_sock.recv(&mut self.response_buffer)?;

		sleep(Duration::from_secs(3));
*/
		self.logger.info("[Tello] Taking off...")?;
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

		self.logger.info("[Tello] Landing gracefully...")?;
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

	fn snapshot(&mut self) -> Option<Arc<DynamicImage>> {
		Some(Arc::new(self.image.clone()?))
	}

	fn rc(&mut self, lr: IUnit, ud: IUnit, fb: IUnit, rot: f32) -> Result<(), Error> {
		debug_assert!(lr >= -100 && lr <= 100);
		debug_assert!(ud >= -100 && ud <= 100);
		debug_assert!(fb >= -100 && fb <= 100);
		debug_assert!(rot >= -100.0 && rot <= 100.0);

		 self.forward_percent = fb as i16;
		 self.sideway_percent = lr as i16;
		 self.rotate_percent = rot as i16;
		 self.updown_percent = ud as i16;
		let pack = packet::set_sticks(self.seq_number, self.forward_percent, self.sideway_percent, self.rotate_percent, self.updown_percent);
		
		self.command_sock.send(&pack)?;
		self.seq_number += 1;

		//self.logger.info_from_string(format!("Just sent out the string: {}", crate::debug_utils::raw_hex_to_string(&pack)))?;

		Ok(())
	}

	fn send_heartbeat(&mut self) -> Result<(), Error> {
		// Only set sticks if we're actively flying.
		match &self.curr_state {
			Some(state) => {
				if state.is_flying {
					dbg!("Sending sticks");
					self.command_sock.send(&set_sticks(self.seq_number, self.rotate_percent, self.updown_percent, self.sideway_percent, self.forward_percent))?;
				}
			}
			None => { /* noop */}
		}
		Ok(())
	}

	fn receive_signal(&mut self, port: u16) -> Result<(), Error> {
		if port == self.command_sock.local_addr()?.port()
		{
			loop {
				let bytes_read = self.command_sock.recv(&mut self.inner_read_buf)?;

				// Nab the last two bytes. In most messages this is the CRC. In the acknowledgement (text) packet, it's the port number.
				//let packet_end = [self.inner_read_buf[bytes_read-2], self.inner_read_buf[bytes_read-1]];

				// This means that it's not formatted like the usual ones, and is probably plain text.
				if self.inner_read_buf[0] != 0xCC {  self.handle_cmd_string(bytes_read)?; }
				else { self.handle_cmd_bytes(bytes_read)?; }
			}
		}
		else if port == self.video_sock.local_addr()?.port()
		{
			self.receive_video()
		}
		else if port == self.command_sock.local_addr()?.port()
		{
			loop {
				let bytes_read = self.command_sock.recv(&mut self.inner_read_buf)?;
				self.logger.info_from_string(format!("[[Info Sock Message]]{}" , crate::debug_utils::raw_hex_to_string(&self.inner_read_buf[..bytes_read])))?;
			}
		}
		else { return Err(Error::Custom("Tello: Requested socket not found in this Tello!")) }
	}

	fn connected(&self) -> bool {
		self.is_connected
	}

	fn time_created(&self) -> SystemTime {
		self.time_created
	}

	fn disconnect(&mut self, ownership_map : &mut HashMap<Token, Connection>) -> Result<(), Error> {
		{
			ownership_map.remove(&Token(self.video_sock.local_addr()?.port() as usize));
			ownership_map.remove(&Token(self.command_sock.local_addr()?.port() as usize));
		}

		{
			let poll_lock = self.poll.lock()?;
			self.video_sock.deregister(poll_lock.registry())?;
			self.command_sock.deregister(poll_lock.registry())?;
		}

		{
			let mut vid_src_lock = self.curr_video_src.lock()?;
			if vid_src_lock.is_some()
			{
				let port = vid_src_lock.as_ref().unwrap().0 as u16;
				if port == self.video_sock.local_addr()?.port()
				{
					*vid_src_lock = None;
				}
			}
		}

		Ok(())
	}

	fn get_state(&self) -> Option<DroneStateJSON> {
		match &self.curr_state {
			None => { None }
			Some(flight_data) => {
				Some(DroneStateJSON::new(
					flight_data.battery_percent,
					420.69, // I don't think I need to say why this is bad...
					SystemTime::now().duration_since(self.time_created).unwrap_or(Duration::new(0, 0)), // FIXME:, not totally accurate, I would actually like to use the data provided by the drone here
					flight_data.height,
					flight_data.is_flying,
					0, // Do we not actually have the signal strength?
					None,
				))
			}
		}
	}
}

impl TelloDrone
{
	#[allow(dead_code)]
	pub(crate) fn new(poll: Arc<Mutex<Poll>>, connection_map: Arc<Mutex<HashMap<Token, Connection>>>, logger : Logger,
					  curr_video_src : Arc<Mutex<Option<Token>>>, curr_video_dst : Arc<Mutex<Option<Token>>>, frame_time : Arc<Duration>,
					  video_queue: VideoQueue,
	) -> Result<Arc<Mutex<Self>>, Error> {
		let mut command_sock = {
			const COMMAND_PORT: u16 = 8889;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, COMMAND_PORT);

			let command_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			command_sock.connect(SocketAddr::V4(SocketAddrV4::new(CONN_ADDR, COMMAND_PORT)))?;

			command_sock
		};

		/*let mut info_sock = {
			const INFO_PORT: u16 = 8890;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, INFO_PORT);
			let info_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			info_sock.connect(SocketAddr::V4(CONN_SOCK))?;

			info_sock
		};*/

		let mut video_sock = {
			const VIDEO_PORT: u16 = 11111;
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, VIDEO_PORT);

			let video_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			//video_sock.connect(SocketAddr::V4(CONN_SOCK))?;

			video_sock
		};

		let com_token = Token(command_sock.local_addr()?.port() as usize);
		let vid_token = Token(video_sock.local_addr()?.port() as usize);

		logger.info_from_string(format!("Command socket opened on: {}",		com_token.0))?;
		logger.info_from_string(format!("Video socket opened on: {}",		vid_token.0))?;

		// Register all the sockets...
		{
			let poll_lock = poll.lock()?;
			let registry = poll_lock.registry();

			registry.register(&mut command_sock,	com_token, Interest::READABLE)?;
			registry.register(&mut video_sock,		vid_token, Interest::READABLE)?;
		}

		let seq_number = 0;
		let response_buffer = vec![0; 255];

		sleep(Duration::from_secs(1));

		let this_drone = Arc::new(Mutex::new(Self {
			command_sock,
			video_sock,
			is_connected	: false,
			time_created	: SystemTime::now(),
			seq_number,
			response_buffer,
			vid_frame_number: 0,
			frame_buffer	: Vec::new(),
			image			: None,
			last_sps_pps_req: std::time::SystemTime::UNIX_EPOCH,	// This prompts an immediate request for SPS/PPS
			sps				: None,
			pps				: None,
			idr				: Vec::new(),
			idr_frame_number: None,
			forward_percent	: 0,
			sideway_percent	: 0,
			rotate_percent	: 0,
			updown_percent	: 0,
			battery_percent	: 0,

			inner_read_buf	: [0;4096],
			logger,
			poll,
			connection_map,
			frame_time,
			last_frame_sent_time: UNIX_EPOCH,
			curr_video_src,
			curr_video_dst,
			curr_state: None,
			video_queue,
		}
		));

		// Add everything to the ownership map, and setup video.
		{
			let mut drone_lock = this_drone.lock()?;
			{
				let mut map_lock = drone_lock.connection_map.lock()?;
				map_lock.insert(com_token, Connection::Drone(this_drone.clone()));
				map_lock.insert(vid_token, Connection::Drone(this_drone.clone()));
			}

			drone_lock.connect()?;

			// FIXME: DEBUG
			*drone_lock.curr_video_src.lock()? = Some(vid_token);
		}

		Ok(this_drone)
	}

	fn request_sps_pss(&mut self) -> Result<(), Error>
	{
		let sps_pps_request = packet::query_video_sps_pps(0);
		self.seq_number += 1;
		self.command_sock.send(&sps_pps_request)?;

		Ok(())
	}

	fn connect(&mut self) -> Result<(), Error>
	{
		const CONN_REQ : [u8;9] = *b"conn_req:";
		let port_bytes : [u8;2] = u16::to_le_bytes(self.video_sock.local_addr()?.port());
		let conn_string : [u8;11] = concat_arrays!(CONN_REQ, port_bytes);
		self.command_sock.send(conn_string.as_bytes())?;

		Ok(())
	}

	fn handle_cmd_bytes(&mut self, bytes_read : usize) -> Result<(), Error>
	{
		let recvd_command : Command = u16::from_le_bytes([self.inner_read_buf[5], self.inner_read_buf[6]]).into();

		match recvd_command {
			Command::Undefined => {
				self.logger.error_from_string(format!("Unexpected message ID from Tello: 0x{:02x} 0x{:02x}\t(msg len: {})\t{}", self.inner_read_buf[5], self.inner_read_buf[6], bytes_read, debug_utils::raw_hex_to_string(&self.inner_read_buf[..bytes_read])))?;
				todo!()
			}
			Command::Error1 => {
				self.logger.warn_from_string(format!("Error1: {}", debug_utils::raw_hex_to_string(&self.inner_read_buf[..bytes_read])))?;
			}
			Command::Error2 => {
				self.logger.warn_from_string(format!("Error2: {}", debug_utils::raw_hex_to_string(&self.inner_read_buf[..bytes_read])))?;
			}
			Command::FlightStatus => {
				let payload = strip_payload(&self.inner_read_buf[..bytes_read]);
				let flight_data = FlightData::new(payload)?;
				// even though we have a bunch of data, looking at it all the time kinda sucks.
				if flight_data.battery_percent != self.battery_percent
				{
					self.battery_percent = flight_data.battery_percent;
					self.logger.info_from_string(format!("Battery Percent: {}", self.battery_percent))?;
				}
			}
			Command::TakeOff => {
				self.logger.info("Tello confirmed: Taking off...")?;
			}
			Command::SetSticks => {
				self.logger.warn("It appears an error occurred when setting the movement.")?;
			}
			Command::Land => {
				self.logger.warn("Tello confirmed: Landing...")?;
			}
			Command::Flip => {
				self.logger.warn("It appears a problem occurred while flipping.")?;
			}
			Command::SetDateTime => {
				self.logger.warn_from_string(format!("SetDateTime packet: {}", debug_utils::raw_hex_to_string(strip_payload(&self.inner_read_buf[..bytes_read]))))?;
			}
			Command::LogHeader => {
				self.logger.warn("Ignoring log header.")?;
			}
			Command::LogData => {
				self.logger.warn("Ignoring log data.")?;
			}
			Command::LogConfig => {
				self.logger.warn("Ignoring log config.")?;
			}
			Command::WifiStatus => {
				// This packet is 13 bytes long.
				let payload = strip_payload(&self.inner_read_buf[..13]);
				if payload.len() != 2 { Err(io::Error::new(io::ErrorKind::InvalidData, "Malformed WifiStatus packet from Tello drone."))? }
				let wifi_percent = payload[0];
				let interference_percent = payload[1];
				if wifi_percent < 70 // arbitrary threshold. I'm just tired of it clogging the logs. TODO: make this a configurable option, or constant, a member perhaps.
				{
					self.logger.info_from_string(format!("Wifi Percent: {}\tInterference level: {}", wifi_percent, interference_percent))?;
				}
			}
			Command::LightStrength => {
				let payload = strip_payload(&self.inner_read_buf[..12]);
				if payload.len() != 1 { Err(io::Error::new(io::ErrorKind::InvalidData, "Malformed LightStrength packet from Tello drone."))? }
				let log = if payload[0] > 0 { "Light level good! "} else { "Light level poor. " };
				self.logger.info(log)?;
			}
			Command::SPSPPS => { todo!("Got back SPSPPS") }
			Command::VideoBitrate => { todo!("Got back VideoBitrate") }
			Command::VideoResolution => { todo!("Got back VideoResolution") }
		}

		Ok(())
	}
	fn handle_cmd_string(&mut self, bytes_read : usize) -> Result<(), Error>
	{
		let message = String::from_utf8_lossy(&self.inner_read_buf[..bytes_read]);
		let packet_end = [self.inner_read_buf[bytes_read - 2], self.inner_read_buf[bytes_read - 1]];

		if message.contains("conn_ack:")
		{
			let port = self.video_sock.local_addr()?.port();
			if port.as_bytes() != packet_end {
				self.logger.error_from_string(format!("Expected acknowledgement: {:04x}\tGot: {:04x}", port, u16::from_ne_bytes(packet_end)))?;
				Err(Error::Custom("Received the wrong value as acknowledgment from Tello."))?
			}
			else {
				self.logger.info("Received handshake acknowledgement from Tello!")?;
				self.is_connected = true;
			}
		}
		else if message.contains("unknown command: ")
		{
			self.logger.warn("We provided an unknown command to Tello.")?;
		}
		else
		{
			println!("no clue what's going on...");
			todo!("Non 0xCC packets from the Tello that weren't acknowledgement. The message: \"{}\"\t{}", message, message.len())
		}

		// TODO: put something here.

		Ok(())
	}

	const SPS_REQUEST_SEC_INTERVAL : Duration = Duration::from_millis(1500);

	fn receive_video(&mut self) -> Result<(), Error>
	{
		// Get image metadata to decode the image.
		let now = SystemTime::now();
		if now.duration_since(self.last_sps_pps_req)? > Self::SPS_REQUEST_SEC_INTERVAL || self.last_sps_pps_req == UNIX_EPOCH
		{
			self.request_sps_pss()?;

			self.last_sps_pps_req = now;
		}

		loop
		{
			const END_OF_FRAME_BITMASK : u8 =  0b1000_0000;
			const PAYLOAD_START : usize = 2;
			// Despite what I thought, it seems that they're using a custom format for these packets.
			// The frame starts [n_frame, 0x00, 0x00, 0x00, 0x00, 0x01, ...] (I suspect the second byte here is also the local number, but it has to be zero.)
			// 	In contrast, the standard H.264 frame starts [0x00, 0x00, 0x01], which means that we can simply strip the first three bytes.
			// The following packets will follow the format [n_frame, n_local, ...], so in theory, we just need to strip that data and we are all set.

			let bytes_read = self.video_sock.recv(&mut self.inner_read_buf)?;
			let frame_number = self.inner_read_buf[0];
			let local_number = self.inner_read_buf[1];
			// Manipulate the internal state of this object's image related members based on the contents of the payload
			// This scope is necessary to let go of the borrow on payload. Of course, we must reborrow this at the end of the function, as we append the payload
			// 	to the current frame_buffer.
			{
				let payload = &self.inner_read_buf[PAYLOAD_START..bytes_read];
				// let nal_type : Option<u8> = ....;
				let end_of_frame = local_number & END_OF_FRAME_BITMASK != 0;

				if bytes_read == 15 && local_number == 0x80 // FIXME: Check the NAL type instead of the length.
				{
					self.logger.info("Received updated SPS (larger)")?;
					self.sps = Some(payload.try_into()?);
				} else if bytes_read == 10 && local_number == 0x80 // FIXME: Check the NAL type instead of the length
				{
					self.logger.info("Received updated PPS (smaller)")?;
					self.pps = Some(payload.try_into()?);
				} else if local_number == 0x80
				{
					todo!("WHAT, THERE'S ANOTHER THING THAT CAN BE 0x80??? {}", payload.len())
				}
				// NAL unit is 0x65, if we're starting an IDR, we should clear all image buffers
				else if payload.len() > 4
				{
					if payload[4] == 0x65 && local_number == 0
					{
						self.idr_frame_number = Some(frame_number);
						self.idr.clear();
						self.frame_buffer.clear();
					}
				}
				// this is basically just asking if it exists yet.
				// TODO: Make this a little cleaner.
				match self.idr_frame_number
				{
					// There's no sense in doing anything with an image if there's no IDR
					None => { continue; }

					Some(n) =>
						{
							// check if we're contributing to the active IDR, if we are, contribute.
							if frame_number == n
							{
								self.idr.extend_from_slice(payload);
								if !end_of_frame
								{
									continue;
								}
							}
						}
				}
			}


			//Check if the frame is ending
			// -- The following is not always true. -- 0x88 is the terminal local number.
			// CLARIFY: When is self.vid_frame_number getting set? Does this make sense?
			if frame_number != self.vid_frame_number && self.frame_buffer.len() > 0
			{

				// Check that we have all the components to output a coherent image.
				if self.sps.is_some() && self.pps.is_some() && self.idr.len() > 0
				{	// The following is for POI.
					//if self.vid_frame_number % 100 == 0
					{
						/*let mut file = File::create(&format!("test_results/frame{frame_number}.h264"))?;
						file.write_all(self.sps.as_ref().unwrap())?;
						file.write_all(self.pps.as_ref().unwrap())?;
						file.write_all(&self.idr)?;
						file.write_all(&self.frame_buffer)?;

						self.logger.info_from_string(format!("Saved a file: {frame_number}\t{}", self.vid_frame_number))?;
						 */
						let mut image_buffer = Vec::new();
						image_buffer.extend_from_slice(&self.sps.unwrap());
						image_buffer.extend_from_slice(&self.pps.unwrap());
						image_buffer.extend_from_slice(&self.idr);
						image_buffer.extend_from_slice(&self.frame_buffer);


						self.video_queue.transcode(Token(self.video_sock.local_addr()?.port() as usize), self.curr_video_src.lock()?.clone(), FrameType::TelloH264, FrameType::Png, image_buffer.into_boxed_slice())?;
					}
					self.vid_frame_number = frame_number;
					self.frame_buffer.clear();
				}
				//if frame_number == 0xFF { panic!("We reached 255.") }
			}


			let payload = &self.inner_read_buf[PAYLOAD_START..bytes_read];
			self.frame_buffer.extend(payload);


		}

		#[allow(unreachable_code)]
		Ok(())
	}

}


impl _DroneInternal for TelloDrone
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
