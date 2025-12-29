use crate::app_network::ClientSocketType::Info;
use crate::drone_interface::Drone;
use crate::ClientSocketType::{Control, Video};
use crate::Result;
use crate::{Connection, Error, ServerInstance, TcpStream};
use image::{DynamicImage, ImageFormat};
use lebe::io::ReadPrimitive;
use mio::Token;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::io::{Cursor, Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zerocopy::IntoBytes;

#[derive(Debug, IntoPrimitive, FromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ClientSocketType
{
	Control = 1,
	Video = 2,
	Info = 3,
	#[num_enum(default)]
	Invalid = 0,
}

#[derive(Debug, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum VideoCode
{
	Jpeg = 26,
	Png = 19,
	#[num_enum(default)]
	Invalid = 0,
}

#[derive(Debug, Clone, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum InfoID
{
	SSIDs = 0,
	DroneStateDump = 1,
	RecordRequest = 2,
	DroneConnectionState = 3,
	DroneSelection = 4,
	#[num_enum(default)]
	Invalid = 255,
}

#[derive(Debug, Clone, FromPrimitive)]
#[repr(u8)]
pub enum RoShamBo
{
	Rock = 0,
	Paper = 1,
	Scissors = 2,
	#[num_enum(default)]
	Invalid = 0xFF,
}

#[derive(Debug, Clone, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum ConnectionState
{
	StillConnecting = 0,
	Connected = 1,
	FailedConnect = 2,
	Disconnected = 3,
	#[num_enum(default)]
	Invalid = 255,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SSIDs
{
	ssids: Vec<String>,
}

#[derive(Debug, Clone, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum CommandCode
{
	Takeoff = 0,
	Land = 1,
	RC = 2,
	#[num_enum(default)]
	Invalid = 255,
}

#[derive(Debug, Clone, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
// CLARIFY: non-exhaustive.
pub enum LandCode
{
	Graceful = 1,
	Emergency = 2,
	#[num_enum(default)]
	Invalid = 255,
}

/// This will only be called when a socket initiates connection.
/// This will not reacquire a lock on the ownership map.
pub fn handle_connection(mut stream : TcpStream,
						 server		: &mut ServerInstance,
						 ownership_map : &mut HashMap<Token, Connection>
) -> Result<()>
{
	// : &mut HashMap<Token, Connection>
	// Arc<Mutex<Option<Token>>>
	let mut handshake_buffer = [0;3]; // Only needs to be 3, then we need to try to read into the buffer again to proc WouldBlock.
	stream.read_exact(&mut handshake_buffer)?;

	if &handshake_buffer[..2] != &[0x42, 0x42] { todo!("Failed to promote socket. Invalid handshake sequence. Received [{:02x} {:02x}]", handshake_buffer[0], handshake_buffer[1])};
	let handshake_code = handshake_buffer[2];

	/*loop {
		match stream.read(&mut handshake_buffer)
		{
			Ok(_) => { server.logger.error("Error while draining read buffer during client-server handshake: There was still data to read from the handshake buffer!")?; }
			Err(e) => {
				if e.kind() == WouldBlock {
					break;
				}
				// Propagate the error if not clean.
				else {
					Err(e)?
				}
			}
		}
	}*/

	let token = Token(stream.local_addr()?.port() as usize);
	let peer_port = stream.peer_addr()?.port() as usize;

	let new_connection =
		match handshake_code.into() {
			ClientSocketType::Control => {
				server.logger.info_from_string(format!("New command socket: {peer_port}"))?;
				server.drone_control = Some(Token(peer_port));
				ownership_map.insert(Token(peer_port), Connection::ClientControl(Control, Arc::new(Mutex::new(stream))));
				handle_control_activity(Token(peer_port), server, ownership_map)?;
			}
			ClientSocketType::Video => {
				server.logger.info_from_string(format!("New video destination: {peer_port}" ))?;
				//*server.video_out.lock()? = Some(Token(peer_port));
				ownership_map.insert(Token(peer_port), Connection::VideoOut(Video, Arc::new(Mutex::new(stream))));
				*server.video_out.lock()? = Some(Token(peer_port));
			}
			ClientSocketType::Info => {
				server.info_token = Some(Token(peer_port));
				server.logger.info_from_string(format!("New bidirectional info stream: {peer_port}"))?;
				// Drain incoming events, in case there are other messages already in queue.
				ownership_map.insert(Token(peer_port), Connection::ServerInfo(Info, Arc::new(Mutex::new(stream))));
				handle_info_activity(Token(peer_port), server, ownership_map)?;
			}

			ClientSocketType::Invalid => { Err(Error::Custom("Invalid socket handshake."))? }
		};

	// Good to note: when we insert a new key-map pair, if the key exists, the value will just be overwritten.
	// Implementation note: right now, the value is being removed from the map anyway, so the above is slightly null.

	Ok(())
}

fn send_image_packet_tcp(out : &mut TcpStream, image_type : VideoCode, image_buffer : &Vec<u8>) -> Result<()>
{
	out.write_all(&(image_type			as u8).to_be_bytes())?; // While this does look ugly, I do like that we have to_be_bytes here. While I don't expect this to ever change, it feels good to just enforce this pattern.
	out.write_all(&(image_buffer.len()	as u16).to_be_bytes())?;
	out.write_all(&image_buffer)?;

	Ok(())
}



pub(crate) fn send_image(
	out				: &mut Connection,
	image			: &DynamicImage,
	image_type		: VideoCode,
) -> Result<()>
{
	let mut image_buffer = Vec::new();
	// Write encoded image data to a buffer. This scoped to avoid mutable and immutable borrows occurring at once.
	{
		let mut image_buffer_writer = Cursor::new(&mut image_buffer);
		match image_type
		{
			VideoCode::Jpeg => {
				image.write_to(&mut image_buffer_writer, ImageFormat::Jpeg)?;
			}
			VideoCode::Png => {
				image.write_to(&mut image_buffer_writer, ImageFormat::Png)?;
			}
			VideoCode::Invalid => { todo!("Unspecified VideoCode, not sure how we should handle this.") }
		};
	}

	match out {
		Connection::ClientControl(_, stream)	=> { send_image_packet_tcp(&mut *stream.lock()?, image_type, &image_buffer) }
		Connection::VideoOut(_, stream)	=> { send_image_packet_tcp(&mut *stream.lock()?, image_type, &image_buffer) }
		Connection::Camera() => { todo!("Haven't implemented this yet.") }
		Connection::UDP(socket) => { debug_assert!(false, "Invalid video target: unpromoted UDP socket {}.", socket.peer_addr()?.port()); Err(Error::NoVideoTarget) }
		Connection::TCP(..) => { debug_assert!(false, "Invalid video target: unpromoted TCP socket."); Err(Error::NoVideoTarget) }
		Connection::Drone(..) => { debug_assert!(false, "Invalid video target: Drone."); Err(Error::NoVideoTarget) }
		Connection::ServerInfo(..) => { debug_assert!(false, "Invalid video target: ServerInfo."); Err(Error::NoVideoTarget) }
	}?;

	Ok(())
}

/// This function just unwraps the tokens and says if we expect the ownership map to hold these values.
/// This does NOT guarantee that we still own the sockets that these ports correspond to.
///
/// The first tuple item is src, the second is out.
#[inline] // doing a borrow of an arc is really stupid, so I hope this gets inlined.
pub(crate) fn _validate_tokens_exist(src : &Option<Token>, out : &Option<Token>) -> Result<(Token, Token)>
{
	let video_out_token = out.as_ref().ok_or(Error::NoVideoTarget)?.clone();
	let video_src_token = src.as_ref().ok_or(Error::NoVideoSource)?.clone();

	Ok((video_src_token, video_out_token))
}

pub(crate) fn handle_info_activity(
	origin	: Token,
	server	: &mut ServerInstance,
	ownership_map : &mut HashMap<Token, Connection>,
) -> Result<()>
{
	if server.info_token.is_none() { todo!("The server did not have an info socket, yet still received activity!") }

	//let peer_port_number = origin.0;
	// We'll bring back this logic if we want to just send this to a work-queue.
	//if origin != *server.info_token.as_ref().unwrap() { todo!("Somehow the info socket did not match the server's info socket? Server expected: {}, Actual: {}", server.info_token.as_ref().unwrap().0, peer_port_number) }

	let info_sock = {
		let inbound_connection = ownership_map.get(&origin);

		let info_sock = match inbound_connection {
			Some(connection) => {
				match connection {
					Connection::ServerInfo(_, stream) => {
						stream
					}
					_ => { Err(Error::Custom("Info socket was NOT the right type of connection."))? }
				}
			}
			// The server doesn't recognize it. This shouldn't happen.
			None => {
				server.logger.error_from_string(format!("The client's info socket was somehow not in the ownership map. Recvd: {}", origin.0))?;
				server.info_token = None;
				Err(Error::Custom("The client's info socket was somehow not in the ownership map. Local: {}, Peer: {}"))?
			}
		};
		info_sock.clone() // This is so the info socket doesn't outlive the ownership map.
	};
	let mut info_lock = info_sock.lock()?;

	loop {
		// Start getting packet info
		#[cfg(debug_assertions)]
		server.logger.info("Attempting to parse incoming info packet")?;
		let read_result = InfoPacket::read(&mut *info_lock);
		match read_result {
			Ok(packet) => {
				#[cfg(debug_assertions)]
				server.logger.info("We received an info packet!")?;
				handle_info_packet(&packet, &mut info_lock, server)?;
			}
			Err(e) => {
				match e {
					Error::IOError(io_error) => {
						match io_error.kind() {
							WouldBlock => { break; }
							_ => { Err(io_error)?; }
						}
					}
					_ => { Err(e)?; }
				}
			}
		};

	}

	Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct InfoPacket
{
	pub id			: InfoID,
	pub play		: RoShamBo,
	//pub payload_size: u16, // this is part of the packet's internal structure, but since this is already encoded in the payload, there is no sense in putting another here.
	pub payload		: Vec<u8>,
}

#[derive(Debug, Clone)]
enum CommandPacket
{
	Takeoff(TakeoffPacket),
	Land(LandPacket),
	RC(RcPacket),
}

// This is basically just a unit... right now.
#[derive(Debug, Clone)]
struct TakeoffPacket
{
	// there is a reserved byte.
}

#[derive(Debug, Clone)]
struct LandPacket
{
	land_code		: LandCode,
}

#[derive(Debug, Clone)]
struct RcPacket
{
	forward			: i8,
	right			: i8,
	up				: i8,
	rot				: i8,
	// there is a reserved byte.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DroneStateJSON
{
	battery			: u8,
	temp			: f32,
	flight_duration	: String,
	height			: u16,
	is_flying		: bool,
	signal_strength	: u8,
	extra			: Option<String>,
}



impl InfoPacket
{
	/// This does assume that the stream is big endian. Git gud if it's not???
	pub fn read<R : Read>(stream : &mut R) -> Result<Self>
	{
		let id : InfoID = u8::read_from_big_endian(stream)?.into(); // again, I know this is redundant for a u8, but whatevs, it's a noop.
		let play : RoShamBo = u8::read_from_big_endian(stream)?.into();
		let payload_size = u16::read_from_big_endian(stream)? as usize;
		let mut payload = vec![0;payload_size];
		stream.read_exact(&mut payload)?;

		Ok(Self {
			id,
			play,
			payload
		})
	}

	pub fn write<W : Write>(&self, stream : &mut W) -> Result<()>
	{
		stream.write_all((self.id.to_owned() as u8).as_bytes())?;
		stream.write_all((self.play.to_owned() as u8).as_bytes())?;
		stream.write_all((self.payload.len() as u16).as_bytes())?;
		stream.write_all(&self.payload)?;

		Ok(())
	}

	pub fn new_ssid(origin_play : RoShamBo, _server : &ServerInstance) -> Result<Self>
	{
		let list_of_ssids = crate::app_network::SSIDs { ssids : vec![String::from_str("Hello")?, String::from_str("world")?, String::from_str("!")?, ] };
		let json = serde_json::to_vec(&list_of_ssids)?;
		// TODO: Make this a feature of the server.
		Ok(Self {
			id : InfoID::SSIDs,
			play: origin_play.counterplay(),
			payload: json,
		})
	}

	pub fn new_drone_connection_state(play : RoShamBo, connection_state: ConnectionState, mac_address : [u8;6]) -> Self
	{
		let mut payload = Vec::new();
		payload.extend_from_slice(&mac_address);
		payload.extend_from_slice((connection_state as u8).as_bytes());
		Self
		{
			id: InfoID::DroneConnectionState,
			play,
			payload,
		}
	}

	/// This may need to be refactored.
	/// This is only sent as a response to the client. Not really sent unsolicited.
	pub fn new_drone_state_dump(origin_play : RoShamBo, mac_address : [u8;6], json : Option<&DroneStateJSON>) -> Result<Self>
	{
		let mut payload = Vec::new();
		payload.extend_from_slice(&mac_address);
		serde_json::to_writer(&mut payload, &Some(&json))?;

		Ok(Self {
			id: InfoID::DroneStateDump,
			play: origin_play.counterplay(),
			payload,
		})
	}
}

impl RoShamBo
{
	pub fn counterplay(&self) -> Self
	{
		match self {
			RoShamBo::Rock		=> { RoShamBo::Paper }
			RoShamBo::Paper		=> { RoShamBo::Scissors }
			RoShamBo::Scissors	=> { RoShamBo::Rock }
			RoShamBo::Invalid	=> { RoShamBo::Invalid } // Invalid is its own counterplay??? Yes.
		}
	}

	pub fn post_increment(&mut self) -> Self
	{
		let result = self.clone();
		*self = self.counterplay();
		result
	}
}

pub(self) fn handle_info_packet(packet : &InfoPacket, origin : &mut TcpStream, server : &mut ServerInstance) -> Result<()>
{
	// FIXME: For the drone stuff, we're using the current drone, but that's not OK.
	match packet.id
	{
		InfoID::SSIDs => { let return_packet = InfoPacket::new_ssid(packet.play.clone(), server)?; return_packet.write(origin)? ; server.logger.error("Sent the client a faux list of SSIDs.") }
		InfoID::DroneStateDump => {
			match &server.curr_drone {
				None => { Ok(()) }
				Some(connection) => {
					match &*connection.lock()?
					{
						Connection::Drone(drone) => {
							let drone_lock = drone.lock()?;
							match drone_lock.get_state() {
								None => {
									InfoPacket::new_drone_state_dump(
										packet.play.counterplay(),
										[0, 0, 0, 0, 0, 0],
										None
									)?;

									Ok(())
								}
								Some(state) => {
									InfoPacket::new_drone_state_dump(
										packet.play.counterplay(),
										[0, 0, 0, 0, 0, 0],
										Some(&state))?;
									Ok(())
								}
							}
						}
						_ => { Err(Error::Custom("While handling info packet, attempting to get drone, the connection was of the wrong type!")) }
					}
				}
			}
		}
		InfoID::RecordRequest => { todo!("Haven't implemented RecordRequest yet.") }
		InfoID::DroneConnectionState => { todo!("Haven't implemented response to DroneConnectionState request yet.") }
		InfoID::DroneSelection => { server.logger.error("We received a DroneSelection packet. This message is a little better than a crash.") }
		InfoID::Invalid => { Err(Error::Custom("Attempted to handle invalid info packet."))? },
	}
}

pub(crate) fn handle_control_activity (
	origin	: Token,
	server: &mut ServerInstance,
	ownership_map : &mut HashMap<Token, Connection>
) -> Result<()>
{
	let peer_port_number = origin.0;
	match server.drone_control
	{
		None => { todo!("The server did not have a client control socket, yet still received activity!") }
		Some(ref drone_control_token) => {
			if origin != *drone_control_token {
				todo!("Somehow the drone control socket did not match the server's info socket? Server expected: {}, Actual: {}", drone_control_token.0, peer_port_number)
			}
		}
	}


	let command_sock = {

		let inbound_connection = ownership_map.get(&origin);

		let command_sock = match inbound_connection {
			Some(connection) => {
				match connection {
					Connection::ClientControl(_, stream) => {
						stream
					}
					_ => { Err(Error::Custom("Drone control socket was NOT the right type of connection."))? }
				}
			}
			// The server doesn't recognize it. This shouldn't happen.
			None => {
				server.drone_control = None;
				Err(Error::Custom("The client's drone control socket was somehow not in the ownership map. "))?
			}
		};
		command_sock.clone() // This is so the command sock doesn't outlive the ownership_map lock.
	};

	let mut command_lock = command_sock.lock()?;

	loop {
		match CommandPacket::read(&mut *command_lock)
		{
			Ok(command_packet) => {
				handle_control_packet(&command_packet, &mut command_lock, server)?
			}
			// This pattern is a bit done to death at this point...
			Err(e) => {
				match e
				{
					Error::IOError(io_error) => {
						match io_error.kind()
						{
							WouldBlock => { break }
							_ => { Err(io_error)? }
						}
					}
					_ => { Err(e)? }
				}
			}
		}
	}

	Ok(())
}

pub(self) fn handle_control_packet(packet : &CommandPacket, origin : &mut TcpStream, server : &mut ServerInstance) -> Result<()>
{
	//TODO: make sure that the right socket is making executive calls here.

	match server.curr_drone
	{
		Some(ref mut connection) => {
			match &mut *connection.lock()?
			{
				Connection::Drone(drone) => {
					match packet {
						CommandPacket::Takeoff(_) => { server.logger.error("Taking off!") }
						CommandPacket::Land(_) => { server.logger.error("Landing!") }
						CommandPacket::RC(_) => { server.logger.error("RC...ing!") }
					}?;

					packet.issue(&mut *drone.lock()?)?
				}
				_ => { todo!("handle_control_packet not a drone."); }
			}

		}
		None => {
			server.logger.warn("Tried to issue a command while not connected to drone!")?;
			return Ok(())
		}
	}

	Ok(())
}

trait Command
{
	fn issue(&self, drone : &mut dyn Drone) -> Result<()>;
}

impl CommandPacket
{
	pub fn read<R : Read>(stream : &mut R) -> Result<Self>
	{
		// Again, the BE is unnecessary, but it's also a noop, so I don't care.
		let command_code: CommandCode = u8::read_from_big_endian(stream)?.into();

		Ok(match command_code
		{
			CommandCode::Takeoff => {
				CommandPacket::Takeoff(TakeoffPacket::read(stream)?)
			}
			CommandCode::Land => {
				CommandPacket::Land(LandPacket::read(stream)?)
			}
			CommandCode::RC => {
				CommandPacket::RC(RcPacket::read(stream)?)
			}
			CommandCode::Invalid => { Err(Error::Custom("Received invalid command code."))? }
		})

	}
}

impl TakeoffPacket
{
	/// This assumes that the COMMAND_CODE has already been consumed.
	pub fn read<R : Read>(stream : &mut R) -> Result<Self>
	{
		let _reserved = u8::read_from_big_endian(stream)?;
		Ok(Self{})
	}

}

impl LandPacket
{
	/// This assumes that the COMMAND_CODE has already been consumed.
	pub fn read<R : Read>(stream : &mut R) -> Result<Self>
	{
		let land_code : LandCode = u8::read_from_big_endian(stream)?.into();
		Ok(Self {
			land_code,
		})
	}
}

impl RcPacket
{
	/// This assumes that the COMMAND_CODE has already been consumed.
	pub fn read<R : Read>(stream : &mut R) -> Result<Self>
	{
		let forward = i8::read_from_big_endian(stream)?.into();
		let right = i8::read_from_big_endian(stream)?.into();
		let up = i8::read_from_big_endian(stream)?.into();
		let rot = i8::read_from_big_endian(stream)?.into();
		let _reserved = u8::read_from_big_endian(stream)?;

		Ok(Self {
			forward,
			right,
			up,
			rot,
		})
	}

}


impl Command for CommandPacket
{
	fn issue(&self, drone: &mut dyn Drone) -> Result<()> {
		match self {
			CommandPacket::Takeoff(takeoff) => { takeoff.issue(drone) }
			CommandPacket::Land(land) => { land.issue(drone) }
			CommandPacket::RC(rc) => { rc.issue(drone) }
		}
	}
}

impl Command for TakeoffPacket
{
	fn issue(&self, drone: &mut dyn Drone) -> Result<()> {
		drone.takeoff()
	}
}

impl Command for LandPacket
{
	fn issue(&self, drone: &mut dyn Drone) -> Result<()> {
		match self.land_code
		{
			LandCode::Graceful => { drone.graceful_land() }
			LandCode::Emergency => { drone.emergency_land() }
			LandCode::Invalid => { Err(Error::Custom("Invalid land code.")) }
		}?;

		Ok(())
	}
}
impl Command for RcPacket
{
	fn issue(&self, drone: &mut dyn Drone) -> Result<()> {
		drone.rc(self.right as i64, self.up as i64, self.forward as i64, self.rot as f32)
	}
}

impl DroneStateJSON
{
	pub fn new(battery : u8, temp : f32, flight_duration : Duration, height : u16, is_flying : bool, signal_strength : u8, extra : Option<String>) -> Self
	{
		let seconds = flight_duration.as_secs();
		let minutes = seconds / 60;
		let hours = minutes / 60;
		let mut flight_duration_string = format!("{:02}:{:02}:{:02}",
												 hours % 24, // Yeah right, a 24 hours flight?
												 minutes % 60,
												 seconds % 60);

		Self {
			battery,
			temp,
			flight_duration : flight_duration_string,
			height,
			is_flying,
			signal_strength,
			extra,
		}
	}

	/*#[allow(dead_code)]
	pub fn read<R : Read>(stream : R) -> Result<Self, Error>
	{
		Self::deserialize(stream)
	}*/

}