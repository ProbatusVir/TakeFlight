use std::collections::HashMap;
use crate::ClientSocketType::Control;
use num_enum::{IntoPrimitive, FromPrimitive};
use std::io::Read;
use std::sync::{Arc, Mutex};
use mio::Token;
use crate::{TcpStream, Error, Connection};
use crate::app_network::ClientSocketType::Video;
//use crate::app_network::ClientSocketType::Control;

#[derive(Debug, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum ClientSocketType
{
	Control = 1,
	Video = 2,
	#[num_enum(catch_all)]
	Invalid(u8),
}

#[derive(Debug, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
enum VideoCode
{
	Jpeg = 26,
	#[num_enum(catch_all)]
	Invalid(u8)
}

/// This will only be called when a socket initiates connection.
pub fn handle_connection(mut stream : TcpStream, ownership_map : &mut HashMap<Token, Connection>) -> Result<crate::Connection, Error>
{
	let mut handshake_buffer = [0;3];
	stream.read_exact(&mut handshake_buffer)?;

	match handshake_buffer[2].into()
	{
		Control => {Ok(Connection::Client(Control, stream))}
		Video => { Ok(Connection::Video(Video, stream)) }
		_ => { Err(Error::Custom("Invalid socket handshake."))? }
	}

}