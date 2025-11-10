use concat_arrays::concat_arrays;
use crate::drone_interface::crc::{crc16, crc8};
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

enum Command
{
	Undefined,
	SetSticks = 0x0050,
	TakeOff = 0x0054,
	Land = 0x0055,
	Flip = 0x005c
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
	let mut header : [u8;9] = [0xCC, 00, 00, 00, 00, 00, 00, 00, 00, ];
	let payload_size : [u8;2] = (payload.len() as u16).to_le_bytes();
	let packet_type_info : u8 = 0b0100_0000 | (packet_type as u8).unbounded_shl(3);
	let message_id : [u8;2] = (command as u16).to_le_bytes();
	let crc = crc8(&header[..3]);

	let sequence_number : [u8;2] = seq.to_le_bytes();
	header[1] = payload_size[0];
	header[2] = payload_size[1];
	header[3] = crc;
	header[4] = packet_type_info;
	header[5] = message_id[0];
	header[6] = message_id[1];
	header[7] = sequence_number[0];
	header[8] = sequence_number[1];

	(header, crc16(payload).to_le_bytes())

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
	let (header, footer) = packet_header(&[], SetInfo, TakeOff, sequence_number);

	concat_arrays!(header, footer)
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

pub fn land(sequence_number : u16) -> [u8;12]
{
	const PAYLOAD : [u8;1] = [0];
	let (header, footer) = packet_header(&PAYLOAD, SetInfo, Land, sequence_number);

	concat_arrays!(header, PAYLOAD, footer)
}

pub fn cancel_land(sequence_number : u16) -> [u8;12]
{
		const PAYLOAD : [u8;1] = [1];
		let (header, footer) = packet_header(&PAYLOAD, SetInfo, Land, sequence_number);

		concat_arrays!(header, PAYLOAD, footer)
}


