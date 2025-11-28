use crate::drone_interface::crc::{crc16_ref, crc8_ref};
use crate::drone_interface::tello::packet::Command::{Land, SetSticks, TakeOff, VideoBitrate, VideoResolution, SPSPPS};
use crate::drone_interface::tello::packet::PacketType::{Data2, GetInfo, SetInfo};
use crate::Error;
use chrono::Timelike;
use concat_arrays::concat_arrays;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::io::{Cursor, Read};
use zerocopy::IntoBytes;

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
	SPSPPS = 0x2500_u16.to_be(),			// This gets the H.264 SPS/PPS. C <-> S.
	VideoBitrate = 0x2800_u16.to_be(),		// This gets the H.264 bitrate. C <-> S.
	VideoResolution = 0x3100_u16.to_be(),
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
pub fn set_sticks(sequence_number : u16, rx : i16, ry : i16, lx : i16, ly : i16) -> [u8;22]
{
	//const MULTIPLE : i16 = i16::MAX / 100;
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

	// The following is taken from [here](https://github.com/Alexander89/rust-tello/blob/93509c63be4008f57b7c8fb77e38efa52f465723/src/lib.rs#L481)
	// Center = 1024. The extremes are ±364
	// TODO: figure this out in more detail.
	//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	/*let rx = (rx as f32) / 100.0;
	let ry = (ry as f32) / 100.0;
	let lx = (lx as f32) / 100.0;
	let ly = (ly as f32) / 100.0;

	let rx = (1024.0 + 660.0 * rx) as i64;
	let ry = (1024.0 + 660.0 * ry) as i64;
	let lx = (1024.0 + 660.0 * lx) as i64;
	let ly = (1024.0 + 660.0 * ly) as i64;
*/
	//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	let rx = 1024 + (66 * rx) / 100;
	let ry = 1024 + (66 * ry) / 100;
	let lx = 1024 + (66 * lx) / 100;
	let ly = 1024 + (66 * ly) / 100;

	let fast = false; // FIXME: This should be a member variable.
	let packed_axes= {
	((rx & 0x07FF)		as i64)
	| (((ry & 0x07FF)	as i64) << 11)
	| (((lx & 0x07FF)	as i64) << 22)
	| (((ly & 0x07FF)	as i64) << 33)
	| (if fast { 1 } else { 0 }) << 44}.to_le_bytes();

	let now = chrono::Local::now();
	let hour = now.hour() as u8;
	let min = now.minute() as u8;
	let sec = now.second() as u8;
	let ms : [u8;2] = ((now.nanosecond() / 1_000_000) as u16).to_le_bytes();

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

pub fn query_video_bitrate(sequence_number : u16) -> [u8;11]
{
	let (footer, header) = packet_header(&[], GetInfo, VideoBitrate, sequence_number);
	concat_arrays!(footer, header)
}

pub fn query_video_sps_pps(sequence_number : u16) -> [u8;11]
{
	let (footer, header) = packet_header(&[], Data2, SPSPPS, sequence_number);
	concat_arrays!(footer, header)
}

/// Ask for future packets to be in 4:3 aspect ratio.
pub fn request_4_3_video(sequence_number : u16) -> [u8;11]
{
	let (footer, header) = packet_header(&[], SetInfo, VideoResolution, sequence_number);
	concat_arrays!(footer, header)
}

#[derive(Debug)]
pub struct FlightData
{
	pub height			: u16, // DECIMETERS???
	pub v_north			: u16,
	pub v_east			: u16,
	pub v_vert			: u16,
	pub fly_time		: u16,

	// Sensor states?
	pub imu_state		: bool,
	pub pressure_state	: bool,
	pub below_state		: bool,
	pub power_state		: bool,
	pub battery_state	: bool,
	pub gravity_state	: bool,
	pub wind_state		: bool,

	pub imu_calibration	: u8,	// not sure.
	pub battery_percent	: u8,
	pub time_remaining	: u16,	// In ms??
	pub battery_mvolts	: u16,

	pub is_flying		: bool,
	pub is_on_ground	: bool,
	pub is_em_open		: bool,	// Electromagnets?? Electrical machinery???
	pub is_hovering		: bool,
	pub outage_recording: bool,
	pub is_battery_low	: bool,
	pub is_battery_crit	: bool,
	pub is_factory_mode	: bool,

	pub fly_mode		: u8,
	pub throw_fly_timer	: u8,
	pub camera_state	: u8,
	pub electrical_machinery_state : u8,

	// I have NO clue what this means...
	pub front_in		: bool,
	pub front_out		: bool,
	pub front_lsc		: bool,
	pub error_state		: bool,
}

impl FlightData
{
	pub fn new(payload : &[u8]) -> Result<Self, Error>
	{
		let mut reader = Cursor::new(payload);
		let mut height : u16 = 0;
		let mut v_north: u16 = 0;
		let mut v_east: u16 = 0;
		let mut v_vert: u16 = 0;
		let mut fly_time : u16 = 0;
		let mut flags1 : u8 = 0;
		let mut imu_calibration: u8 = 0;
		let mut battery_percent: u8 = 0;
		let mut time_remaining: u16 = 0;
		let mut battery_mvolts: u16 = 0;
		let mut flags2 : u8 = 0;
		let mut fly_mode : u8 = 0;
		let mut throw_fly_timer : u8 = 0;
		let mut camera_state : u8 = 0;
		let mut electrical_machinery_state : u8 = 0;
		let mut flags3 : u8 = 0; // there's only 3 in this one.
		let mut flags4 : u8 = 0; // There's only 1 in this one.
		{
			reader.read(height.as_mut_bytes())?;
			reader.read(v_north.as_mut_bytes())?;
			reader.read(v_east.as_mut_bytes())?;
			reader.read(v_vert.as_mut_bytes())?;
			reader.read(fly_time.as_mut_bytes())?;
			reader.read(flags1.as_mut_bytes())?;
			reader.read(imu_calibration.as_mut_bytes())?;
			reader.read(battery_percent.as_mut_bytes())?;
			reader.read(time_remaining.as_mut_bytes())?;
			reader.read(battery_mvolts.as_mut_bytes())?;
			reader.read(flags2.as_mut_bytes())?;
			reader.read(fly_mode.as_mut_bytes())?;
			reader.read(throw_fly_timer.as_mut_bytes())?;
			reader.read(camera_state.as_mut_bytes())?;
			reader.read(electrical_machinery_state.as_mut_bytes())?;
			reader.read(flags3.as_mut_bytes())?;
			reader.read(flags4.as_mut_bytes())?;
		}

		let imu_state				= flags1 & 0b0000_0001 != 0;	// 0
		let pressure_state		= flags1 & 0b0000_0010 != 0;	// 1
		let below_state			= flags1 & 0b0000_0100 != 0;	// 2
		let power_state			= flags1 & 0b0000_1000 != 0;	// 3
		let battery_state			= flags1 & 0b0001_0000 != 0;	// 4
		let gravity_state			= flags1 & 0b0010_0000 != 0;	// 5 -- 6 is unknown.
		let wind_state			= flags1 & 0b1000_0000 != 0;	// 7

		let is_flying				= flags2 & 0b0000_0001 != 0;	// 0
		let is_on_ground			= flags2 & 0b0000_0010 != 0;	// 1
		let is_em_open			= flags2 & 0b0000_0100 != 0;	// 2
		let is_hovering			= flags2 & 0b0000_1000 != 0;	// 3
		let outage_recording		= flags2 & 0b0001_0000 != 0;	// 4
		let is_battery_low		= flags2 & 0b0010_0000 != 0;	// 5
		let is_battery_crit		= flags2 & 0b0100_0000 != 0;	// 6
		let is_factory_mode		= flags2 & 0b1000_0000 != 0;	// 7

		let front_in				= flags3 & 0b0000_0001 != 0;	// 0
		let front_out				= flags3 & 0b0000_0010 != 0;	// 1
		let front_lsc				= flags3 & 0b0000_0100 != 0;	// 2

		let error_state = flags4 == 1;

		Ok(Self {
			height,
			v_north,
			v_east,
			v_vert,
			fly_time,
			imu_state,
			pressure_state,
			below_state,
			power_state,
			battery_state,
			gravity_state,
			wind_state,
			imu_calibration,
			battery_percent,
			time_remaining,
			battery_mvolts,
			is_flying,
			is_on_ground,
			is_em_open,
			is_hovering,
			outage_recording,
			is_battery_low,
			is_battery_crit,
			is_factory_mode,
			fly_mode,
			throw_fly_timer,
			camera_state,
			electrical_machinery_state,
			front_in,
			front_out,
			front_lsc,
			error_state,
		})
	}
}
