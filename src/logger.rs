use crate::error::Error;
use chrono::{Local, Timelike};
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, };
use std::thread;

pub(crate) enum LoggingLevel
{
	Info,
	Warning,
	Error,
}

pub(crate) struct LogMessage
{
	pub logging_level: LoggingLevel,
	pub time : chrono::DateTime<Local>,
	pub msg : String,
	// I'll see if I like this...
	pub thread : String,
}

/// Only make one of these.
#[derive(Clone, Debug)]
pub struct Logger
{
	sender : Sender<LogMessage>,
}

impl Logger
{
	/// Please only make one of these.
	pub fn new() -> (Self, Receiver<LogMessage>)
	{
		let (sender, receiver) = std::sync::mpsc::channel();
		(Self { sender }, receiver)
	}

	pub fn info(&self, msg : &str) -> Result<(), Error>
	{
		self.send_log_message(LoggingLevel::Info, msg).map_err(|_| Error::Custom("Unable send INFO message to logger!"))
	}

	pub fn warn(&self, msg : &str) -> Result<(), Error>
	{
		self.send_log_message(LoggingLevel::Warning, msg).map_err(|_| Error::Custom("Unable send WARN message to logger!"))
	}

	// It makes sense that, for the most, part, we want more descriptive error messages.
	#[allow(dead_code)]
	pub fn error(&self, msg : &str) -> Result<(), Error>
	{
		self.send_log_message(LoggingLevel::Error, msg).map_err(|_| Error::Custom("Unable send ERROR message to logger!"))
	}

	pub fn error_from_string(&self, msg : String) -> Result<(), Error>
	{
		self.send_log_message_string(LoggingLevel::Error, msg)
	}

	pub fn warn_from_string(&self, msg : String) -> Result<(), Error>
	{
		self.send_log_message_string(LoggingLevel::Warning, msg)
	}

	pub fn info_from_string(&self, msg : String) -> Result<(), Error>
	{
		self.send_log_message_string(LoggingLevel::Info, msg)
	}

	fn send_log_message(&self, logging_level: LoggingLevel, msg : &str) -> Result<(), Error>
	{
		let msg = String::from_str(msg)?;
		self.send_log_message_string(logging_level, msg)
	}

	fn send_log_message_string(&self, logging_level: LoggingLevel, msg : String) -> Result<(), Error>
	{
		let time = chrono::Local::now();
		let thread = thread::current().name().unwrap().into();
		self.sender.send(LogMessage { logging_level, time, msg, thread}).map_err(|_| Error::Custom("Failed to send message to logger!"))
	}
}

/// I would love to make this return Result<!, Error> once it becomes stable.
pub fn do_logging(receiver: Receiver<LogMessage>, log_file : Arc<Mutex<Option<File>>>, continue_logger : Arc<Mutex<bool>>) -> Result<(), Error>
{
	let mut continue_loop = true;
	while *continue_logger.lock()? {
		// Receive our message, but make sure that we actually have one.
		let log_message = match receiver.recv() {
				Ok(message) => { message }
				Err(error) => {
					let error_message = error.to_string();
					continue_loop = false;
					LogMessage
					{
						logging_level: LoggingLevel::Error,
						time: chrono::Local::now(),
						msg: "Error receiving messages. Did 'main' panic? Shutting down logger.".to_string(),
						thread: thread::current().name().unwrap().into(), // I'd be very surprised if this was uninitialized somehow...
					}
				}
			};

		// Format our message
		let message_out = {
				format!("[{}]({:02}:{:02}:{:02}): [{}] \"{}\"",
						match log_message.logging_level {
							LoggingLevel::Info => { "INFO" }
							LoggingLevel::Warning => { "WARN" }
							LoggingLevel::Error => { "ERR " }
						},
						log_message.time.hour(),
						log_message.time.minute(),
						log_message.time.second(),
						log_message.thread,
						log_message.msg,
				)
			};

		// Actually write out the message
		match log_message.logging_level {
			LoggingLevel::Info		=> {  println!("{message_out}"); }
			LoggingLevel::Warning	=> { eprintln!("{message_out}"); }
			LoggingLevel::Error		=> { eprintln!("{message_out}"); }
		}

		{
			let mut log_file_lock = log_file.lock().map_err(|_| Error::Custom("Logger unable to get a lock on the log_file pointer!"))?;

			match &mut *log_file_lock
			{
				Some(log_file_lock) => { log_file_lock.write(format!("{message_out}\n").as_bytes())?; }
				None => {}
			}
		}
		if !continue_loop { Err(Error::Custom("Error receiving messages. Did 'main' panic? Shutting down logger."))? }
	}
	
	Ok(())
}
