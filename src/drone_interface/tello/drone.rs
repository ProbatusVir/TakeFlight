use super::packet;
use crate::drone_interface::tello::packet::{land, set_sticks, strip_payload, Command, FlightData};
use crate::drone_interface::{Drone, IUnit, Unit};
use crate::error::Error;
use crate::logger::Logger;
use crate::video::rtp::RTPHeader;
use crate::{debug_utils, drone_interface, send_image, Connection, Poll, Token, UdpSocket};
use concat_arrays::concat_arrays;
use const_format::concatcp;
use image::DynamicImage;
use mio::Interest;
use openh264::formats::YUVSource;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Cursor, Write};
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
	info_sock		: UdpSocket,

	seq_number		: u16,
	response_buffer	: Vec<u8>,
	vid_frame_number: u8,
	frame_buffer	: Vec<u8>,
	image			: Option<DynamicImage>,

	last_sps_pps_req: std::time::SystemTime,
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
		match &self.image
		{
			Some(img) => { Some(Arc::new(img.as_bytes().to_vec())) }
			None => { None }
		}
	}

	fn rc(&mut self, lr: IUnit, ud: IUnit, fb: IUnit, rot: f32) -> Result<(), Error> {
		debug_assert!(lr >= -100 && lr <= 100);
		debug_assert!(ud >= -100 && ud <= 100);
		debug_assert!(fb >= -100 && fb <= 100);
		debug_assert!(rot >= -100.0 && rot <= 100.0);

		let pack = packet::set_sticks(self.seq_number, lr as i16, rot as i16, fb as i16, ud as i16);

		self.command_sock.send(&pack)?;
		self.seq_number += 1;

		self.logger.info_from_string(format!("Just sent out the string: {}", crate::debug_utils::raw_hex_to_string(&pack)))?;

		Ok(())
	}

	// FIXME: This is for debug purposes *only*
	#[cfg(debug_assertions)]
	fn send_heartbeat(&mut self) -> Result<(), Error> {
		static mut DEBUG_NUMBER : usize = 0;

		let debug_number = unsafe { DEBUG_NUMBER };

		if debug_number == 0 {
			self.takeoff()?;
		}
		else if debug_number > 20 {
			self.graceful_land()?;
		}
		else if debug_number > 10 {
			self.rotate_percent = 50;
			self.logger.info_from_string(format!("Rotation: {}", self.rotate_percent))?;
			self.command_sock.send(&set_sticks(self.seq_number, self.rotate_percent, self.updown_percent, self.sideway_percent, self.forward_percent))?;
		}
		else {
			self.rotate_percent = 0;
			self.logger.info_from_string(format!("rotation: {}\tvertical: {}\tsideways: {}\tforward: {}", self.rotate_percent, self.updown_percent, self.sideway_percent, self.forward_percent))?;
			self.command_sock.send(&set_sticks(self.seq_number, self.rotate_percent, self.updown_percent, self.sideway_percent, self.forward_percent))?;
		}

		unsafe {
			DEBUG_NUMBER += 1;
		}

		self.seq_number += 1;

		Ok(())
	}

	#[cfg(not(debug_assertions))]
	fn send_heartbeat(&mut self) -> Result<(), Error> {
		self.command_sock.send(&set_sticks(self.seq_number, self.rotate_percent, self.updown_percent, self.sideway_percent, self.forward_percent))?;

		Ok(())
	}

	fn receive_signal(&mut self, port: u16) -> Result<(), Error> {
		if port == self.command_sock.local_addr()?.port()
		{
			loop {
				let bytes_read = self.command_sock.recv(&mut self.inner_read_buf)?;

				// Nab the last two bytes. In most messages this is the CRC. In the acknowledgement (text) packet, it's the port number.
				let packet_end = [self.inner_read_buf[bytes_read-2], self.inner_read_buf[bytes_read-1]];

				// This means that it's not formatted like the usual ones, and is probably plain text.
				if self.inner_read_buf[0] != 0xCC {  self.handle_cmd_string(bytes_read)?; }
				else { self.handle_cmd_bytes(bytes_read)?; }
			}
		}
		else if port == self.video_sock.local_addr()?.port()
		{
			self.receive_video()
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
	pub(crate) fn new(poll: Arc<Mutex<Poll>>, connection_map: Arc<Mutex<HashMap<Token, Connection>>>, logger : Logger,
					  curr_video_src : Arc<Mutex<Option<Token>>>, curr_video_dst : Arc<Mutex<Option<Token>>>, frame_time : Arc<Duration>,
	) -> Result<Arc<Mutex<Self>>, Error> {
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
			const CONN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1);
			const CONN_SOCK: SocketAddrV4 = SocketAddrV4::new(CONN_ADDR, VIDEO_PORT);

			let video_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))?;
			//video_sock.connect(SocketAddr::V4(CONN_SOCK))?;

			video_sock
		};

		let com_token = Token(command_sock.local_addr()?.port() as usize);
		let vid_token = Token(video_sock.local_addr()?.port() as usize);
		let nfo_token = Token(info_sock.local_addr()?.port() as usize);

		logger.info_from_string(format!("Command socket connected to: {}",		com_token.0))?;
		logger.info_from_string(format!("Video socket connected to: {}",		vid_token.0))?;
		logger.info_from_string(format!("Info socket connected to: {}",		nfo_token.0))?;

		// Register all the sockets...
		{
			let poll_lock = poll.lock()?;
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
		}
		));

		// Add everything to the ownership map, and setup video.
		{
			let mut drone_lock = this_drone.lock()?;
			{
				let mut map_lock = drone_lock.connection_map.lock()?;
				map_lock.insert(com_token, Connection::Drone(this_drone.clone()));
				map_lock.insert(vid_token, Connection::Drone(this_drone.clone()));
				map_lock.insert(nfo_token, Connection::Drone(this_drone.clone()));
			}

			drone_lock.connect()?;
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
				self.logger.info("Taking off...")?;
			}
			Command::SetSticks => {
				self.logger.warn("It appears an error occurred when setting the movement.")?;
			}
			Command::Land => {
				self.logger.warn("Landing...")?;
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
			// Despite what I thought, it seems that they're using a custom format for these packets.
			// The frame starts [n_frame, 0x00, 0x00, 0x00, 0x00, 0x01, ...] (I suspect the second byte here is also the local number, but it has to be zero.)
			// 	In contrast, the standard H.264 frame starts [0x00, 0x00, 0x01], which means that we can simply strip the first three bytes.
			// The following packets will follow the format [n_frame, n_local, ...], so in theory, we just need to strip that data and we are all set.

			let bytes_read = self.video_sock.recv(&mut self.inner_read_buf)?;
			let frame_number = self.inner_read_buf[0];
			let local_number = self.inner_read_buf[1];
			let payload_start = 2; // FIXME: this is literally a numerical constant, empirically this number holds up. Once we finalize, just write 2.
			let payload = &self.inner_read_buf[payload_start..bytes_read];
			// let nal_type : Option<u8> = ....;
			let end_of_frame = local_number & END_OF_FRAME_BITMASK != 0;

			if bytes_read == 15 && local_number == 0x80 // FIXME: Check the NAL type instead of the length.
			{
				self.logger.info("Received updated SPS (larger)")?;
				self.sps = Some(payload.try_into()?);
			}
			else if bytes_read == 10 && local_number == 0x80 // FIXME: Check the NAL type instead of the length
			{
				self.logger.info("Received updated PPS (smaller)")?;
				self.pps = Some(payload.try_into()?);
			}
			else if local_number == 0x80
			{
				todo!("WHAT, THERE'S ANOTHER THING THAT CAN BE 0x80??? {}", payload.len())
			}
			// NAL unit is 0x65, if we're starting an IDR, we should clear all image buffers
			else if payload.len() >= 4
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



			//Check if the frame is ending
			// -- The following is not always true. -- 0x88 is the terminal local number.
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


						let mut decoder = openh264::decoder::Decoder::new()?;
						let decoder_result = decoder.decode(&image_buffer);
						match decoder_result
						{
							Ok(decoded_option) =>
							{
								/*
								let decoded = decoded_option.unwrap();
								let (w,h) = decoded.dimensions();
								self.logger.info_from_string(format!("We successfully decoded frame {frame_number}, {w}x{h}"))?;
								let mut file = File::create(format!("test_results/frame{frame_number}.rgb"))?;
								decoded.write_rgb8(&mut file_buffer);
								file.write_all(&file_buffer)?;*/

								// Send the image to the client, if possible.
								// TODO: I think this can be optimized for space if we initialize our image once, etc. etc.
								let decoded = decoded_option.unwrap();
								let (w,h) = decoded.dimensions();
								let mut decoded_image = image::RgbImage::new(w as u32, h as u32);
								decoded.write_rgb8(&mut decoded_image);
								self.image = Some(decoded_image.into());

								let now = SystemTime::now();
								if now.duration_since(self.last_frame_sent_time)? >= *self.frame_time
								{
									match send_image(self.curr_video_dst.clone(), self.curr_video_src.clone(), self.connection_map.clone())
									{
										Err(Error::NoVideoSource) => { }
										Err(Error::NoVideoTarget) => { }
										Ok(_) => {  }
										e => { e? }
									}
								}
							}
							Err(_) =>
							{ 	//We actually don't really care about this error.
								//self.logger.error_from_string(format!("Received malformed video frame {frame_number}"))?;
								//File::create(format!("test_results/malformed{frame_number}"))?.write_all(&image_buffer)?
							}
						}
					}
					self.vid_frame_number = frame_number;
					self.frame_buffer.clear();
				}
				//if frame_number == 0xFF { panic!("We reached 255.") }
			}

			self.frame_buffer.extend(payload);


		}

		Ok(())
	}

}