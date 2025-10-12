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
