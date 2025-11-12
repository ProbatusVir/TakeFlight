use crate::ClientSocketType::{Control, Video};
use crate::{Connection, Error, ServerInstance, TcpStream};
use mio::Token;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::io::Read;

#[derive(Debug, IntoPrimitive, FromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ClientSocketType
{
	Control = 1,
	Video = 2,
	#[num_enum(catch_all)]
	Invalid(u8),
}

#[derive(Debug, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
enum VideoCode
{
	Jpeg = 26,
	#[num_enum(catch_all)]
	Invalid(u16)
}

/// This will only be called when a socket initiates connection.
/// This will not reacquire a lock on the ownership map.
pub fn handle_connection(mut stream : TcpStream, server : &mut ServerInstance) -> Result<Connection, Error>
{
	let mut handshake_buffer = [0;3];
	stream.read_exact(&mut handshake_buffer)?;

	let token = Token(stream.local_addr()?.port() as usize);

	let new_connection =
		match handshake_buffer[2].into() {
			Control => {
				server.drone_control = Some(token);
				Connection::Client(Control, stream)
			}
			Video => {
				server.video_out = Some(token);
				Connection::VideoOut(Video, stream)
			}
			_ => { Err(Error::Custom("Invalid socket handshake."))? }
		};

	// Good to note: when we insert a new key-map pair, if the key exists, the value will just be overwritten.
	// Implementation note: right now, the value is being removed from the map anyway, so the above is slightly null.

	Ok(new_connection)
}