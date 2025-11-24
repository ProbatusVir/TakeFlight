use std::collections::HashMap;
use crate::ClientSocketType::{Control, Video};
use crate::{Connection, Error, ServerInstance, TcpStream};
use mio::Token;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::io::{ Cursor, Read, Write};
use std::sync::{Arc, Mutex};
use image::{DynamicImage, ImageFormat};
use mio::net::UdpSocket;

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
#[repr(u8)]
pub enum VideoCode
{
	Jpeg = 26,
	Png = 19,
	#[num_enum(default)]
	Invalid = 0,
}

/// This will only be called when a socket initiates connection.
/// This will not reacquire a lock on the ownership map.
pub fn handle_connection(mut stream : TcpStream,
						 server		: &mut ServerInstance,
) -> Result<Connection, Error>
{
	// : &mut HashMap<Token, Connection>
	// Arc<Mutex<Option<Token>>>
	let mut handshake_buffer = [0;3];
	stream.read_exact(&mut handshake_buffer)?;

	let token = Token(stream.local_addr()?.port() as usize);

	let new_connection =
		match handshake_buffer[2].into() {
			Control => {
				server.drone_control = Some(token);
				Connection::Client(Control, Arc::new(Mutex::new(stream)))
			}
			Video => {
				let peer_port = stream.peer_addr()?.port() as usize;
				server.logger.info_from_string(format!("New video destination: {peer_port}" ))?;
				*server.video_out.lock()? = Some(Token(peer_port));
				Connection::VideoOut(Video, Arc::new(Mutex::new(stream)))
			}
			_ => { Err(Error::Custom("Invalid socket handshake."))? }
		};

	// Good to note: when we insert a new key-map pair, if the key exists, the value will just be overwritten.
	// Implementation note: right now, the value is being removed from the map anyway, so the above is slightly null.

	Ok(new_connection)
}

fn send_image_packet_tcp(out : &mut TcpStream, image_type : VideoCode, image_buffer : &Vec<u8>) -> Result<(), Error>
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
) -> Result<(), Error>
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
		Connection::Client(_, stream)	=> { send_image_packet_tcp(&mut *stream.lock()?, image_type, &image_buffer) }
		Connection::VideoOut(_, stream)	=> { send_image_packet_tcp(&mut *stream.lock()?, image_type, &image_buffer) }
		Connection::Camera() => { todo!("Haven't implemented this yet.") }
		Connection::Drone(_) => { debug_assert!(false, "Invalid video target: Drone."); Err(Error::NoVideoTarget) }
		Connection::UDP(socket) => { debug_assert!(false, "Invalid video target: unpromoted UDP socket."); Err(Error::NoVideoTarget) }
		Connection::TCP(_) => { debug_assert!(false, "Invalid video target: unpromoted TCP socket."); Err(Error::NoVideoTarget) }
	}?;

	Ok(())
}

/// This function just unwraps the tokens and says if we expect the ownership map to hold these values.
/// This does NOT guarantee that we still own the sockets that these ports correspond to.
///
/// The first tuple item is src, the second is out.
#[inline] // doing a borrow of an arc is really stupid, so I hope this gets inlined.
pub(crate) fn _validate_tokens_exist(src : &Option<Token>, out : &Option<Token>) -> Result<(Token, Token), Error>
{
	let video_out_token = out.as_ref().ok_or(Error::NoVideoTarget)?.clone();
	let video_src_token = src.as_ref().ok_or(Error::NoVideoSource)?.clone();

	Ok((video_src_token, video_out_token))
}

/*#[allow(dead_code)]
pub(crate) fn send_image_server_edition(
	out				: Arc<Mutex<Option<Token>>>,
	src				: Arc<Mutex<Option<Token>>>,
	ownership_map	: Arc<Mutex<HashMap<Token, Connection>>>,
) -> Result<(), Error>
{
	// While this is a large critical section, I actually think it's for the best, due to all the validations and possible reassignments of our streams.
	let mut video_out = out.lock()?;
	let mut video_src = src.lock()?;

	let (video_src_token, video_out_token) = _validate_tokens_exist(&video_src, &video_out)?;

	let mut ownership_lock = ownership_map.lock()?;
	let src = ownership_lock.remove(&video_src_token).ok_or_else(|| {
		*video_src = None;
		Error::NoVideoSource
	})?;
	let mut out = ownership_lock.remove(&video_out_token).ok_or_else(|| {
		*video_out = None;
		Error::NoVideoTarget
	})?;

	let image = match &src {
		Connection::Drone(source) => {
			{
				// FIXME: THIS IS WHERE THE DEADLOCK IS HAPPENING!
				let mut source_lock = source.lock()?;
				source_lock.snapshot().clone().ok_or(Error::Custom("Could not obtain image from drone!"))?.clone()
			}

		}
		Connection::Camera() => todo!(),
		_ => { dbg!("Oh, we're silly billies who forgot how our own enumerator worked???"); Err(Error::NoVideoSource)? }
	};

	match out
	{
		Connection::VideoOut(cnx_type, ref mut stream) => {
			let mut stream_lock = stream.lock()?;
			stream_lock.write(&[u8::from(cnx_type.clone())])?;
			stream_lock.write(&(image.len() as u16).to_be_bytes())?;
			stream_lock.write_all(&image)?
		}
		_ => { Err(Error::NoVideoTarget)? }
	}

	ownership_lock.insert(video_src_token, src);
	ownership_lock.insert(video_out_token, out);

	Ok(())
}*/