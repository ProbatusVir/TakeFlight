use rusqlite as sql;
use crate::error::Error;
pub const DB_DIR : &str = "logs/flightdb.db3";

#[test]
fn db_connect_test() -> Result<(), Error>
{
	let db = sql::Connection::open(DB_DIR)?;

	Ok(())
}

fn test() -> Result<(), Error>
{
	let mut db = sql::Connection::open(DB_DIR)?;

	Ok(())
}
