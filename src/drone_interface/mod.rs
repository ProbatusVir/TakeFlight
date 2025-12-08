pub mod tello;

#[allow(dead_code)] // Doing this because we are going to deprecate this soon.
pub mod drone_pro;
mod crc;

use std::collections::HashMap;
use crate::app_network::{send_image, VideoCode};
use crate::{Connection, Error, ServerMap};
use image::DynamicImage;
use mio::net::UdpSocket;
use mio::Token;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// The unit corresponds to a centimeter, for now; even if the precision of the drone is not matched to the centimeter.

#[allow(dead_code)]
pub type Unit = u64;
pub type IUnit = i64;

// These are sorted by chronological order... sort of.
#[derive(Debug, Eq, PartialEq)]
pub enum ConnectionState
{
	Connected,
	StillConnecting,
	FailedConnect,
	Disconnected,
}

#[allow(dead_code)]
pub trait Drone : Debug
{
	/// The init will establish whatever internal state is necessary.
	/// This must include whatever network operations are necessary
	/// for communicating with the drone. This may ***NOT*** include
	/// takeoff.
	///
	/// Due to the nature of the registry and map, any drone must clean
	/// up its entries in both the map and registry.
	///
	/// ⚠ INIT HAS BEEN DEPRECATED -- THESE CONSTRAINTS SHOULD APPLY TO THE DRONE'S INITIALIZATION METHODS ⚠

	/// The drone will reach an operational height.
	fn takeoff(&mut self) -> Result<(), Error>;
	/// The drone will immediately cease all motor activity, this
	/// should also power down the drone and disconnect all network
	/// activity to the drone.

	fn emergency_land(&mut self) -> Result<(), Error>;

	/// The drone will attempt to land stably. No guarantees on
	/// precisely where the drone will land.
	fn graceful_land(&mut self) -> Result<(), Error>;

	/// Will move the drone x centimeters upward.
	fn up(&mut self, x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters downward.
	fn down(&mut self, x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters in the direction
	/// it is facing.
	fn forward(&mut self, x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters opposite of the
	/// direction it is facing.
	fn backward(&mut self, x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters left of the
	/// direction it is facing.
	fn left(&mut self, x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters right of the
	/// direction it is facing.
	fn right(&mut self, x : Unit) -> Result<(), Error>;

	fn backflip(&mut self) -> Result<(), Error>;

	fn frontflip(&mut self) -> Result<(), Error>;

	/// The drone's yaw may be adjusted in radians.
	/// A negative radian will result in a counter
	/// clockwise rotation.
	fn clockwise_rot(&mut self, rads: f32) -> Result<(), Error>;

	/// The drone's yaw may be adjusted in radians.
	/// A negative radian will result in a clockwise
	/// rotation.
	fn cclockwise_rot(&mut self, rads : f32) -> Result<(), Error>;

	/// Will return a picture from the drone's video feed.
	fn snapshot(&mut self) -> Option<Arc<DynamicImage>>;


	/// The drone may be free to move on all axes simultaneously.
	/// Relative to the drone's forward vector, these commands follow:
	///
	/// lr : left/right
	///
	/// ud : up/down
	///
	/// fb : forward/backward
	///
	/// rot : rotation about the yaw plane
	///
	/// There is no guarantee that these transformations must occur at the same time.
	fn rc(&mut self, lr: IUnit, ud : IUnit, fb :  IUnit, rot :  f32) -> Result<(), Error>;

	fn send_heartbeat(&mut self) -> Result<(), Error>;
	fn receive_signal(&mut self, port : u16) -> Result<(), Error>;

	fn connection_state(&self) -> ConnectionState;
	fn time_created(&self) -> SystemTime;
	fn disconnect(&mut self, ownership_map : &mut HashMap<Token, Connection>) -> Result<(), Error>;
}

pub(crate) trait _DroneInternal : Drone
{
	/// Does not acquire lock or anything. Importantly, this does not *borrow* from the Drone.
	fn expose_video_stream_port(&self) -> Result<u16, Error>;
	/// Does not acquire lock or anything. Importantly, this does not *borrow* from the Drone.
	#[allow(dead_code)]
	fn expose_video_stream(&mut self) -> &mut UdpSocket;

	/// Does not acquire lock or anything. Importantly, this does not *borrow* from the Drone.
	fn expose_ownership_map(&self) -> ServerMap;
	/// Does not acquire lock or anything. Importantly, this does not *borrow* from the Drone.
	fn expose_server_src_token(&self) -> Arc<Mutex<Option<Token>>>;

	/// Does not acquire lock or anything. Importantly, this does not *borrow* from the Drone.
	fn expose_server_out_token(&self) -> Arc<Mutex<Option<Token>>>;

	/// Actual utility to send images.
	fn send_image(&mut self, video_code : VideoCode, ) -> Result<(), Error>
	{
		let own_vid_token = Token(self.expose_video_stream_port()? as usize);

		let src = self.expose_server_src_token();
		let out = self.expose_server_out_token();

		// While this is a large critical section, I actually think it's for the best, due to all the validations and possible reassignments of our streams.
		let mut video_src = src.lock()?;
		let mut video_out = out.lock()?;

		let (video_src_token, video_out_token) = crate::app_network::_validate_tokens_exist(&video_src, &video_out)?;

		// Make sure that we are the source (or not), and that we are NOT the destination.
		debug_assert_ne!(video_out.unwrap_or(Token(0)), own_vid_token);    // Just make sure it's not possible for our drone to be receiving our video.
		if own_vid_token != video_src_token { return Err(Error::NoVideoSource); }

		// Get the relevant sockets if they're still valid, or make sure they are unregistered.
		// TODO: We should investigate whether we also need to unregister these from the registry.
		let ownership_map = self.expose_ownership_map();
		let mut ownership_lock = ownership_map.lock()?;
		let _src = ownership_lock.get_mut(&video_src_token).ok_or_else(|| {
			*video_src = None;
			Error::NoVideoSource
		})?;
		let mut out = ownership_lock.get_mut(&video_out_token).ok_or_else(|| {
			*video_out = None;
			Error::NoVideoTarget
		})?;

		// The requested format will override the one specified by the drone.
		// FIXME: This is more of a shortcoming of our own network specification.
		/*
		match out {
			Connection::Client(_, _) => { video_code = format }
			Connection::VideoOut(_, _) => { video_code = format }
			_ => { /* noop */ }
		}*/

		let image = self.snapshot();

		match image
		{
			Some(img) => { send_image(&mut out, &img, video_code) } // not this trait's `send_image`.
			None => { Err(Error::NoVideoSource) }
		}
	}

}