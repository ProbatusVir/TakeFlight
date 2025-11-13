use chrono::Timelike;
use concat_arrays::concat_arrays;
use lebe::Endian;
use zerocopy::IntoBytes;
use crate::drone_interface::crc::{crc16, crc16_ref, crc8, crc8_ref};
use crate::drone_interface::tello::packet::Command::{Land, SetSticks, TakeOff};
use crate::drone_interface::tello::packet::PacketType::{Data2, SetInfo};
use crate::UdpSocket;
use crate::Error;
use num_enum::{Default, FromPrimitive, IntoPrimitive};

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
#[derive(FromPrimitive, IntoPrimitive)]
pub enum Command
{
	#[default()]
	Undefined,

	Error1 = 0x4300_u16.to_be(),
	Error2 = 0x4400_u16.to_be(),
	SetDateTime = 0x4600_u16.to_be(),
	SetSticks = 0x5000_u16.to_be(),
	TakeOff = 0x5400_u16.to_be(),
	Land = 0x5500_u16.to_be(),
	FlightStatus = 0x5600_u16.to_be(),
	Flip = 0x5c00_u16.to_be(),
	LogHeader = 0x5010_u16.to_be(),
	LogData = 0x5110_u16.to_be(),
	LogConfig = 0x5210_u16.to_be(),
	WifiStatus = 0x1a00_u16.to_be(),
	LightStrength = 0x3500_u16.to_be(),
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
}

#[test]
fn test_takeoff()
{
	let seq = u16::from_big_endian_into_current(0x5003);
	let result = takeoff(seq);
	assert_eq!(result, [0xcc, 0x58, 0x00, 0x7c, 0x68, 0x54, 0x00, 0x50, 0x03, 0xde, 0x68]);
}

// Each value must be between 0-100.
pub fn set_sticks(sequence_number : u16, mut rx : i16, mut ry : i16, mut lx : i16, mut ly : i16) -> [u8;22]
{
	const MULTIPLE : i16 = i16::MAX / 100;
	debug_assert!(rx >= 0 && rx < 100);
	debug_assert!(ry >= 0 && ry < 100);
	debug_assert!(lx >= 0 && lx < 100);
	debug_assert!(ly >= 0 && ly < 100);

	// I wish SIMD wasn't just nightly...
	/*rot	*= MULTIPLE;
	ud	*= MULTIPLE;
	lr	*= MULTIPLE;
	fb	*= MULTIPLE;
	 */

	let packed_axes= {
	((rx & 0x07FF)	as i64)
	| (((ry & 0x07FF)	as i64) << 11)
	| (((lx & 0x07FF)	as i64) << 22)
	| (((ly & 0x07FF)	as i64) << 33)}.to_be_bytes();



	let now = chrono::Local::now();
	/*let hour = now.hour() as u8;
	let min = now.minute() as u8;
	let sec = now.second() as u8;
	let ms : [u8;2] = ((now.nanosecond() / 1_000_000) as u16).to_le_bytes();*/
	let hour = 0;
	let min = 0;
	let sec = 0;
	let ms = [0, 0];

	let payload : [u8;11] = {[
		packed_axes[0],
		packed_axes[1],
		packed_axes[2],
		packed_axes[3],
		packed_axes[4],
		packed_axes[5],
		hour,
		min,
		sec,
		ms[0],
		ms[1]
	]};


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

/// Zero-sized payloads are OK
pub fn strip_payload(packet : &[u8]) -> &[u8]
{
	debug_assert!(packet.len() >= 11);
	&packet[9..packet.len() - 2]
}

pub struct FlightData
{
	height			: u16, // DECIMETERS???
	v_north			: u16,
	v_east			: u16,
	v_vert			: u16,
	fly_time		: u16,

	// Sensor states?
	imu_state		: bool,
	pressure_state	: bool,
	below_state		: bool,
	power_state		: bool,
	battery_state	: bool,
	gravity_state	: bool,
	wind_state		: bool,

	imu_calibration	: u8,	// not sure.
	battery_percent	: u8,
	time_remaining	: u16,	// In ms??
	battery_mvolts	: u16,

	is_flying		: bool,
	is_on_ground	: bool,
	is_em_open		: bool,	// Electromagnets?? Electrical machinery???
	is_hovering		: bool,
	outage_recording: bool,
	is_battery_low	: bool,
	is_battery_crit	: bool,
	is_factory_mode	: bool,

	fly_mode		: u8,
	throw_fly_timer	: u8,
	camera_state	: u8,
	electrical_machinery_state : u8,

	// I have NO clue what this means...
	front_in		: bool,
	front_out		: bool,
	front_lsc		: bool,
	error_state		: bool,
}

impl FlightData
{
	pub fn new(payload : &[u8])
	{

	}
}
