use crate::error::Error;
use chrono::{Local, Timelike};
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, };

enum LoggingLevel
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
	pub fn new() -> (Logger, Receiver<LogMessage>)
	{
		let (sender, receiver) = std::sync::mpsc::channel();
		(Self { sender }, receiver)
	}

	pub fn info(&self, msg : String) -> Result<(), Error>
	{
		self.send_log_message(LoggingLevel::Info, msg).map_err(|_| Error::Custom("Unable send INFO message to logger!"))
	}

	pub fn warn(&self, msg : String) -> Result<(), Error>
	{
		self.send_log_message(LoggingLevel::Warning, msg).map_err(|_| Error::Custom("Unable send WARN message to logger!"))
	}

	pub fn error(&self, msg : String) -> Result<(), Error>
	{
		self.send_log_message(LoggingLevel::Error, msg).map_err(|_| Error::Custom("Unable send ERROR message to logger!"))
	}

	fn send_log_message(&self, logging_level: LoggingLevel, msg : String) -> Result<(), Error>
	{
		let time = chrono::Local::now();

		self.sender.send(LogMessage { logging_level, time, msg, }).map_err(|_| Error::Custom("Failed to send message to logger!"))
	}
}

pub fn do_logging(receiver: Receiver<LogMessage>, log_file : Arc<Mutex<Option<File>>>) -> Result<(), Error>
{
	loop {
		// Receive our message
		let log_message = receiver.recv().map_err(|_| Error::Custom("Logger failed!"))?;

		// Format our message
		let message_out = {
				format!("[{}] ({}:{}:{}): \"{}\"",
						match log_message.logging_level {
							LoggingLevel::Info => { "INFO" }
							LoggingLevel::Warning => { "WARN" }
							LoggingLevel::Error => { "ERR" }
						},
						log_message.time.hour(),
						log_message.time.minute(),
						log_message.time.second(),
						log_message.msg,
				)
			};

		// Actually write out the message
		println!("{message_out}");
		{
			let mut log_file_lock = log_file.lock().map_err(|_| Error::Custom("Logger unable to get a lock on the log_file pointer!"))?;

			match &mut *log_file_lock
			{
				Some(log_file_lock) => { log_file_lock.write(format!("{message_out}\n").as_bytes())?; }
				None => {}
			}
		}
	}
}
