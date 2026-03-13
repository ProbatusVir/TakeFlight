mod entry;

use crate::error::Error;
use rusqlite as sql;
use rusqlite::{named_params, params, Connection, OptionalExtension};

pub const DB_DIR : &str = "logs/flightdb.db3";
pub const SCRIPTS_DIR : &str = "src/sql_scripts";
#[test]
fn db_connect_test() -> Result<(), Error>
{
	let db = sql::Connection::open(DB_DIR)?;

	Ok(())
}

/// This table is independent.
#[test]
fn test_insert_gesture() -> Result<(), Error>
{
	let mut db = sql::Connection::open(DB_DIR)?;

	// Don't think too hard about this one... It takes in arbitrary float arrays.
	entry::insert_row_gesture_control(&mut db, &[1.0, 2.0, 3.0], "flip left")?;

	Ok(())
}

/// This table is independent
/// In production we'll either be using an enum (prolly not...), or we'll get the SDK with some select with a table that doesn't exist yet.
/// The SDK-Drone mapping is probably going to be user defined when they set up the drone, if there is no way to detect for that drone.
/// Presently, the drones that we work with all identify themselves with two letters at the start of their SSID. -- this isn't the place to document this...
/// In the future, we might use a hash as a key for this drone_data table, where their MAC address is hashed to a 64 bit key.
#[test]
fn test_insert_drone_data() -> Result<(), Error>
{
	let mut db = sql::Connection::open(DB_DIR)?;

	// Don't think too hard about this one... It takes in arbitrary float arrays.
	entry::insert_row_drone_data(&mut db, 42)?;

	Ok(())
}

#[test]
fn test_insert_flight_model() -> Result<(), Error>
{
	// Arrange
	let mut db = Connection::open(DB_DIR)?;
	ensure_drone_data(&mut db)?;

	// Act
	/// Again, there will be semantic data to this. This is just poor practice.
	entry::insert_row_flight_model(&mut db, 1)?;

	Ok(())
}

#[test]
fn test_insert_flight_log() -> Result<(), Error>
{
	// Arrange
	let mut db = Connection::open(DB_DIR)?;
	ensure_drone_data_and_gesture(&mut db)?;

	// Act
	entry::insert_row_flight_log(&mut db, 1, 2, (3.0, 4.0, 5.0), 6.0, 7.0, chrono::Utc::now())?;

	Ok(())

}

#[cfg(test)]
fn ensure_drone_data_and_gesture(db : &mut Connection) -> Result<(), Error>
{
	ensure_drone_data(db)?;

	if db.query_row("SELECT * FROM GestureControlTb", [], |row| { row.get::<usize, String>(0) }).optional()?.is_none()
	{
		match entry::insert_row_gesture_control(db, &[1.0, 2.0, 3.0], "THIS IS STRICTLY FOR TESTING")
		{
			Ok(_) => {}
			Err(_) => { panic!("We did not test the intended method. This failed because we did not have gesture data.") }
		}
	}

	Ok(())
}

#[cfg(test)]
fn ensure_drone_data(db : &mut Connection) -> Result<(), Error>
{
	if db.query_row("SELECT * FROM DroneDataTb", [], |row| { row.get::<usize, u64>(0) }).optional()?.is_none()
	{
		match entry::insert_row_drone_data(db, 1)
		{
			Ok(_) => {}
			Err(_) => { panic!("We did not test the intended method. This failed because we did not have drone data.") }
		}
	}

	Ok(())
}
