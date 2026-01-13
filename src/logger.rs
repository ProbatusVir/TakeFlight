use crate::error::Error;
use crate::Result;
use chrono::{Local, Timelike};
use std::fmt::{Display, Formatter};
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
	thread : String,
	logging_level: LoggingLevel,
	time : chrono::DateTime<Local>,
	msg : String,
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

	pub fn info(&self, msg : &str) -> Result<()>
	{
		self.send_log_message(LoggingLevel::Info, msg).map_err(|_| Error::Custom("Unable send INFO message to logger!"))
	}

	pub fn warn(&self, msg : &str) -> Result<()>
	{
		self.send_log_message(LoggingLevel::Warning, msg).map_err(|_| Error::Custom("Unable send WARN message to logger!"))
	}

	// It makes sense that, for the most, part, we want more descriptive error messages.
	#[allow(dead_code)]
	pub fn error(&self, msg : &str) -> Result<()>
	{
		self.send_log_message(LoggingLevel::Error, msg).map_err(|_| Error::Custom("Unable send ERROR message to logger!"))
	}

	pub fn error_from_string(&self, msg : String) -> Result<()>
	{
		self.send_log_message_string(LoggingLevel::Error, msg)
	}

	pub fn warn_from_string(&self, msg : String) -> Result<()>
	{
		self.send_log_message_string(LoggingLevel::Warning, msg)
	}

	pub fn info_from_string(&self, msg : String) -> Result<()>
	{
		self.send_log_message_string(LoggingLevel::Info, msg)
	}

	fn send_log_message(&self, logging_level: LoggingLevel, msg : &str) -> Result<()>
	{
		let msg = String::from_str(msg)?;
		self.send_log_message_string(logging_level, msg)
	}

	fn send_log_message_string(&self, logging_level: LoggingLevel, msg : String) -> Result<()>
	{
		self.sender.send(LogMessage::new(logging_level, msg)).map_err(|_| Error::Custom("Failed to send message to logger!"))
	}
}

/// I would love to make this return Result<!, Error> once it becomes stable.
pub fn do_logging(receiver: Receiver<LogMessage>, log_file : Arc<Mutex<Option<File>>>, continue_logger : Arc<Mutex<bool>>) -> Result<()>
{
	// showoff at the start
	{
		const BUILD : &str = env!("BUILD");
		write_message_out(LogMessage::new(LoggingLevel::Info, "Logger started!"), &log_file)?;
		write_message_out(LogMessage::new(LoggingLevel::Info, format!("Running version: {BUILD}")), &log_file)?;
	}
	while *continue_logger.lock()? {
		// Receive our message, but make sure that we actually have one.
		match receiver.recv() {
			Ok(message) => {
				write_message_out(message, &log_file)?
			}
			Err(error) => {
				const ERROR_MESSAGE : &str = "Error receiving messages. Did 'main' panic? Shutting down logger.";
				write_message_out(LogMessage::new(LoggingLevel::Error, ERROR_MESSAGE), &log_file)?;
				break;
			} // Err
		} // match
	}

	let final_message = LogMessage::new(LoggingLevel::Info, "Shutting down!");
	write_message_out(final_message, &log_file)?;

	Ok(())
}

fn write_message_out(message : LogMessage, log_file : &Arc<Mutex<Option<File>>>) -> Result<()>
{
	match message.logging_level {
		LoggingLevel::Info		=> {  println!("{message}")}
		LoggingLevel::Warning	=> { eprintln!("{message}")}
		LoggingLevel::Error		=> { eprintln!("{message}")}
	}

	{
		let mut log_file_lock = log_file.lock().map_err(|_| Error::Custom("Logger unable to get a lock on the log_file pointer!"))?;

		match &mut *log_file_lock
		{
			Some(log_file_lock) => { writeln!(log_file_lock, "{message}")?; }
			None => { /* noop */ }
		}

		Ok(())
	}
}


impl LogMessage
{
	/// Sealed.
	fn new<T : ToString>(logging_level : LoggingLevel, msg : T) -> Self
	{
		/*
		pub logging_level: LoggingLevel,
		pub time : chrono::DateTime<Local>,
		pub msg : String,
		// I'll see if I like this...
		pub thread : String,
		 */
		let time = chrono::Local::now();
		let thread = thread::current().name().unwrap_or_default().into();

		Self
		{
			logging_level,
			time,
			msg : msg.to_string(),
			thread,
		}
	}
}

impl Display for LogMessage
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}]({:02}:{:02}:{:02}): [{}] \"{}\"",
			match self.logging_level {
				LoggingLevel::Info		=> { "INFO" }
				LoggingLevel::Warning	=> { "WARN" }
				LoggingLevel::Error		=> { "ERR " }
			},
			self.time.hour(),
			self.time.minute(),
			self.time.second(),
			self.thread,
			self.msg,
		)
	}
}