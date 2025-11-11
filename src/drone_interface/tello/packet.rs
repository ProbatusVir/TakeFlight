use concat_arrays::concat_arrays;
use lebe::Endian;
use zerocopy::IntoBytes;
use crate::drone_interface::crc::{crc16, crc16_ref, crc8, crc8_ref};
use crate::drone_interface::tello::packet::Command::{Land, SetSticks, TakeOff};
use crate::drone_interface::tello::packet::PacketType::{Data2, SetInfo};
use crate::UdpSocket;
use crate::Error;

#[allow(dead_code)]
const HEADER : u8 = 0xCC;

#[allow(dead_code)]
// These values are all three bits
enum PacketType
{
	Extended	= 0x00,
	GetInfo		= 0x01,
	Data1		= 0x02,
	Unknown1	= 0x03,
	Data2		= 0x04,
	SetInfo		= 0x05,
	Flip		= 0x06,
	Unknown2	= 0x07,
}

#[repr(u16)]
enum Command
{
	Undefined,
	SetSticks = 0x5000_u16.to_be(),
	TakeOff = 0x5400_u16.to_be(),
	Land = 0x5500_u16.to_be(),
	Flip = 0x5c00_u16.to_be(),
}

#[allow(dead_code)]
struct Packet
{
	message : Vec<u8>
}

/// The payload is only used for its size.
/// The first result is the header, the second is the crc at the end.
fn packet_header(payload : &[u8], packet_type: PacketType, command: Command, seq : u16) -> ([u8;9], [u8;2])
{
	let mut header: [u8; 9] = [0xCC, 00, 00, 00, 00, 00, 00, 00, 00, ];
	let payload_size: [u8; 2] = packet_size(payload.len() as u16);
	let packet_type_info: u8 = 0b0100_0000 | (packet_type as u8).unbounded_shl(3);
	let message_id: [u8; 2] = (command as u16).to_le_bytes();

	let sequence_number: [u8; 2] = seq.to_le_bytes();
	header[1] = payload_size[0];
	header[2] = payload_size[1];
	let crc = crc8_ref(&header[..3]);
	header[3] = crc;
	header[4] = packet_type_info;
	header[5] = message_id[0];
	header[6] = message_id[1];
	header[7] = sequence_number[0];
	header[8] = sequence_number[1];

	let last_crc: [u8; 2] = crc16_ref(header.iter().chain(payload.iter())).to_le_bytes();

	(header, last_crc)
}
/*fn create_full_packet<const N : usize>(payload : [u8;N], sequence_number : u16) -> [u8;N + 11]
{
	todo!()
}*/


// For movement components, they will be sent as a u16.
// Speed (0-100, discrete) * 327. The value 327 is
// seemingly from i16::MAX / 100 truncated.

pub fn takeoff(sequence_number : u16) -> [u8;11]
{
	// I want to make it this simple.
	let (header, footer) = packet_header(&[], SetInfo, TakeOff, sequence_number);
	concat_arrays!(header, footer)


	// this implementation works right now.
	/*let size = packet_size::<0>();
	let seq : [u8;2] = sequence_number.to_le_bytes();


	let mut array = [
		0xCC,				// Constant
		size[0], size[1],	// Size of whole packet
		0,					// CRC-8
		0x68,				// Packet type
		(TakeOff as u16).as_bytes()[0], (TakeOff as u16).as_bytes()[1],			// Message ID
		seq[0], seq[1],		// Sequence
		0, 0,				// CRC-16
	];

	let header_and_count : [u8;3] = array[..3].try_into().unwrap();
	let first_crc = crc8(header_and_count);
	array[3] = first_crc;

	let array_clone : [u8;9] = array[..9].try_into().unwrap();
	let last_crc : [u8;2] = crc16(array_clone).to_le_bytes();
	array[9] = last_crc[0];
	array[10] = last_crc[1];

	array*/
}

#[test]
fn test_takeoff()
{
	let seq = u16::from_big_endian_into_current(0x5003);
	let result = takeoff(seq);
	assert_eq!(result, [0xcc, 0x58, 0x00, 0x7c, 0x68, 0x54, 0x00, 0x50, 0x03, 0xde, 0x68]);
}

// Each value must be between 0-100.
pub fn set_sticks(sequence_number : u16, mut rot : i16, mut ud : i16, mut lr : i16, mut fb : i16) -> [u8;19]
{
	const MULTIPLE : i16 = i16::MAX / 100;
	debug_assert!(rot > 0 && rot < 100);
	debug_assert!(ud > 0 && ud < 100);
	debug_assert!(lr > 0 && lr < 100);
	debug_assert!(fb > 0 && fb < 100);

	// I wish SIMD wasn't just nightly...
	rot *= MULTIPLE;
	ud *= MULTIPLE;
	lr *= MULTIPLE;
	fb *= MULTIPLE;

	let payload : [u8;8] = concat_arrays!(rot.to_le_bytes(), ud.to_le_bytes(), lr.to_le_bytes(), fb.to_le_bytes());
	let (header, footer) = packet_header(&payload, Data2, SetSticks, sequence_number);

	concat_arrays!(header, payload, footer)
}

/// This also does the endian conversion
pub const fn packet_size(payload_len : u16) -> [u8;2]
{
	((payload_len + 11) << 3).to_le_bytes()
}



pub fn land(sequence_number : u16) -> [u8;12]
{
	const PAYLOAD : [u8;1] = [0x00];
	let (header, footer) = packet_header(&PAYLOAD, SetInfo, Land, sequence_number);

	/*[
		0xCC,		// Constant
		0x58, 0x00,	// Size of whole packet - the first byte, little endian, after its shifted 3 to the right.
		0x7c, 0x68,
		0x54, 0x00
	]*/

	concat_arrays!(header, PAYLOAD, footer)
}

#[test]
fn test_land()
{
	let seq = 0x2a06_u16.from_big_endian_into_current();
	let result = land(seq);
	assert_eq!(result, [0xcc_u8, 0x60, 0x00, 0x27, 0x68, 0x55, 0x00, 0x2a, 0x06, 0x00, 0xef, 0xca]);
}


pub fn cancel_land(sequence_number : u16) -> [u8;12]
{
		const PAYLOAD : [u8;1] = [1];
		let (header, footer) = packet_header(&PAYLOAD, SetInfo, Land, sequence_number);

		concat_arrays!(header, PAYLOAD, footer)
}

