pub mod tello;

#[allow(dead_code)] // Doing this because we are going to deprecate this soon.
pub mod drone_pro;
mod crc;

use crate::Error;
use std::fmt::Debug;
use std::sync::Arc;

/// The unit corresponds to a centimeter, for now; even if the precision of the drone is not matched to the centimeter.

#[allow(dead_code)]
pub type Unit = u64;
pub type IUnit = i64;

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
	fn snapshot(&mut self) -> Option<Arc<Vec<u8>>>;


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
}