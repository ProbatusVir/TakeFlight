use const_format::formatcp;
use zerocopy::{Immutable, IntoBytes};

#[allow(dead_code)]
pub(crate) fn view_raw_memory<Any : IntoBytes + Immutable + ?Sized>(any : &Any)
{
	let _ = any.as_bytes().into_iter().for_each(|byte| { print!("{byte:08b}") });
}

#[allow(dead_code)]
pub(crate) fn view_raw_hex<Any : IntoBytes + Immutable + ?Sized>(any : &Any)
{
	let _ = any.as_bytes().into_iter().for_each(|byte| { print!("{byte:02x} ") });
	println!();
}

#[allow(dead_code)]
pub(crate) fn raw_memory_to_string<Any : IntoBytes + Immutable + ?Sized>(any : &Any) -> String
{
	let mut result = String::new();
	let _ = any.as_bytes().into_iter().for_each(|byte| { result.push_str(&format!("{byte:08b} ")) });

	result
}

#[allow(dead_code)]
pub(crate) fn raw_hex_to_string<Any : IntoBytes + Immutable + ?Sized>(any : &Any) -> String
{
	let mut result = String::new();
	let _ = any.as_bytes().into_iter().for_each(|byte| { result.push_str(&format!("{byte:02x} ")) });

	result
}
