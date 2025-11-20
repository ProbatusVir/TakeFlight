// ⚠ LOTS OF MACROS AHEAD ⚠
// ⚠ NOT FOR THE FAINT OF ⚠
// ⚠       HEART!!!       ⚠

use super::{named_params, params, Connection};
use crate::Error;
use chrono::{DateTime, Utc};
use serde::Serialize;


pub fn insert_row_drone_data(db : &mut Connection,  sdk : u64) -> Result<(), Error>
{
	const CMD : &str = include_str!(concat!(env!("SQL_ENTRY_DIR"), "/drone_data_tb.sql"));

	db.execute(&CMD, [sdk])?;

	Ok(())
}


pub fn insert_row_gesture_control(db : &mut Connection, key_points : &[f32], mcro : &str) -> Result<(), Error>
{
	const CMD : &str = include_str!(concat!(env!("SQL_ENTRY_DIR"), "/gesture_control_tb.sql"));

	let mut serializer = serde_json::Serializer::new(Vec::new());
	key_points.serialize(&mut serializer)?;
	let stringified_keypoints = String::from_utf8(serializer.into_inner())?;


	db.execute(&CMD, params![&stringified_keypoints, mcro])?;

	Ok(())
}

/// It should be noted, that the System Time is not monotonic, and all the weirdness that entails. So just don't check the times...
pub fn insert_row_flight_log(db : &mut Connection, flight_id : u64, gesture_id : u64, velocity : (f32, f32, f32), pitch : f32, yaw : f32, entry_time : DateTime<Utc>) -> Result<(), Error>
{
	const CMD : &str = include_str!(concat!(env!("SQL_ENTRY_DIR"), "/flight_log_tb.sql"));

	db.execute(&CMD, named_params!{
		":FlightID":	flight_id,
		":VelocityX":	velocity.0,
		":VelocityY":	velocity.1,
		":VelocityZ":	velocity.2,
		":Pitch":		pitch,
		":Yaw":			yaw,
		":GestureID":	gesture_id,
		":EntryTime":	entry_time
	})?;

	Ok(())
}

/// When a flight starts, this should be the first entry
pub fn insert_row_flight_model(db : &mut Connection, flight_id : u64, drone_model_id : u64) -> Result<(), Error>
{
	const CMD : &str = include_str!(concat!(env!("SQL_ENTRY_DIR"), "/flight_model_tb.sql"));

	db.execute(&CMD, params![drone_model_id])?;

	Ok(())
}