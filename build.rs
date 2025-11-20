use std::io::{Read, };
use std::path;
use anyhow::Error;
use const_format::{formatcp};

const TEST_RESULT_FOLDER : &str = "test_results";
const LOGS_FOLDER : &str = "logs";
const FLIGHT_DB : &str = formatcp!("{LOGS_FOLDER}/flightdb.db3");
const SQL_SCRIPTS : &str = "src/sql_scripts";
const TABLE_SCRIPTS : &str = "src/sql_scripts/create_tables";

fn main() -> Result<(), Error>
{

	if !std::fs::exists(TEST_RESULT_FOLDER)? { std::fs::create_dir(TEST_RESULT_FOLDER)?; }

	if !std::fs::exists(LOGS_FOLDER)? { std::fs::create_dir(LOGS_FOLDER)?; }

	if !std::fs::exists(FLIGHT_DB)? { setup_database()? }

	let path_buf = path::absolute(SQL_SCRIPTS)?;	// scripts_dir is a reference to this, so we need a binding to keep it alive... boring...
 	let scripts_dir = path_buf.to_str().unwrap();
	println!("cargo::rustc-env=TABLE_SCRIPTS={scripts_dir}/create_tables");
	println!("cargo::rustc-env=SCRIPTS_DIR={}", scripts_dir);
	println!("cargo::rustc-env=SQL_ENTRY_DIR={scripts_dir}/entry");

	Ok(())
}

fn setup_database() -> Result<(), Error>
{
	const DRONE_DATA : &str = formatcp!("{TABLE_SCRIPTS}/drone_data_tb.sql");
	const FLIGHT_LOG : &str = formatcp!("{TABLE_SCRIPTS}/flight_log_tb.sql");
	const FLIGHT_MODEL : &str = formatcp!("{TABLE_SCRIPTS}/flight_model_tb.sql");
	const GESTURE_CTRL : &str = formatcp!("{TABLE_SCRIPTS}/gesture_control_tb.sql");
	println!("Creating database");
	let db = rusqlite::Connection::open(FLIGHT_DB)?;

	let mut read_buffer = String::new();

	println!("Creating drone data table from script at '{DRONE_DATA}'.");
	let mut drone_data_table = std::fs::File::open(DRONE_DATA)?;
	drone_data_table.read_to_string(&mut read_buffer)?;
	read_buffer += ";\n";

	println!("Creating flight model table from script at '{FLIGHT_MODEL}'.");
	let mut flight_model_table = std::fs::File::open(FLIGHT_MODEL)?;
	flight_model_table.read_to_string(&mut read_buffer)?;
	read_buffer += ";\n";

	println!("Creating gesture control table from script at '{GESTURE_CTRL}'.");
	let mut gesture_control_table = std::fs::File::open(GESTURE_CTRL)?;
	gesture_control_table.read_to_string(&mut read_buffer)?;
	read_buffer += ";\n";

	println!("Creating flight log table from script at '{FLIGHT_LOG}'.");
	let mut flight_log_table = std::fs::File::open(FLIGHT_LOG)?;
	flight_log_table.read_to_string(&mut read_buffer)?;
	read_buffer += ";\n";

	db.execute_batch(&read_buffer)?;

	Ok(())
}
