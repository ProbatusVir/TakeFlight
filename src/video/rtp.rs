use crate::video::rtp::BitFlagsVPXCCMPT::{ShiftCC, ShiftM, ShiftP, ShiftPT, ShiftV, ShiftX, CC, M, P, PT, V, X};
use crate::error::Error;
use lebe::io::ReadPrimitive;
use std::io::Read;
use zerocopy::IntoBytes;

pub struct RTPHeader
{
	pub version				: u8,
	pub payload_type		: u8,
	pub sequence_number		: u16,
	pub timestamp			: u32,
	pub ssrc				: u32,
	pub is_last_in_frame	: bool,
}

#[repr(u16)]
enum BitFlagsVPXCCMPT
{
	V	= 0b1100_0000__0000_0000_u16.to_be(),
	P	= 0b0010_0000__0000_0000_u16.to_be(), // binary
	X	= 0b0001_0000__0000_0000_u16.to_be(), // binary
	CC	= 0b0000_1111__0000_0000_u16.to_be(),
	M	= 0b0000_0000__1000_0000_u16.to_be(), // binary
	PT	= 0b0000_0000__0111_1111_u16.to_be(),

	ShiftV	= 0_u16,
	ShiftP	= 1_u16,
	ShiftX	= 2_u16,
	ShiftCC	= 3_u16,
	ShiftM	= 7_u16,
	ShiftPT	= 8_u16,
}

impl RTPHeader
{
	pub fn from_stream(mut stream: impl Read) -> Result<Self, Error>
	{
		let mut flags : u16 = 0;
		let mut sequence_number : u16 = 0;
		let mut timestamp : u32 = 0;
		let mut ssrc : u32 = 0;
		//let mut csrcs : u32 = 0;
		{
			stream.read(flags.as_mut_bytes())?;
			stream.read(sequence_number.as_mut_bytes())?;
			stream.read(timestamp.as_mut_bytes())?;
			stream.read(ssrc.as_mut_bytes())?;
			//stream.read(csrcs.as_mut_bytes())?;
		}

		let version = ((flags & V as u16) >> ShiftV as u16) as u8 >> (u8::BITS - (V as u16).count_ones());
		assert_eq!(version, 2); // Mandatory, per the standard
		let padding = (flags & P as u16) >> ShiftP as u16;
		let extension = (flags & X as u16) >> ShiftX as u16;
		let csrc_count = ((flags & CC as u16) >> ShiftCC as u16) as u8;
		let marker = (flags & M as u16) >>  ShiftM as u16;
		let payload_type = ((flags & PT as u16) >> ShiftPT as u16) as u8;

		assert_eq!(extension, 0, "Extensions not implemented!");
		assert_eq!(padding, 0, "Padding not implemented!");
		assert_eq!(csrc_count, 0, "CSRCs not implemented!");
		//assert_eq!(marker, 0, "Markers not implemented!");
		let is_last_in_frame = marker != 0;

		Ok(Self {
			version,
			payload_type,
			sequence_number,
			timestamp,
			ssrc,
			is_last_in_frame,
		})
	}
}

#[derive(Debug)]
pub struct JpegMainHeader
{
	pub type_specific		: u8,
	pub fragment_offset		: u32,
	pub packet_type 		: u8,
	pub quantization		: u8,
	pub width				: u16,
	pub height				: u16,
	pub restart_header		: Option<JpegRestartHeader>,
	pub quantization_header	: Option<JpegQuantizationTableHeader>
}


impl JpegMainHeader
{
	pub fn is_image_start(&self) -> bool
	{
		let quant_header_exists = match self.quantization_header {
			Some(_)	=> { true }
			None	=> { false }
		};

		quant_header_exists
	}

	pub fn from_stream(mut stream : impl Read, ignore_quant : bool) -> Result<Self, Error>
	{
		let width : u16;
		let height : u16;
		let mut restart_header = None;
		let mut quantization_header = None;
		// I'm not sure this is accurate...
		let type_specific	: u8;
		let mut fragment_offset	: u32 = 0; // u24, more like.
		let packet_type		: u8;
		let quantization	: u8;
		let unscaled_width	: u8;
		let unscaled_height	: u8;

		type_specific = u8::read_from_big_endian(&mut stream)?;
		// A complex case
		stream.read(&mut (fragment_offset.as_mut_bytes()[0..3]))?;
		fragment_offset = u32::from_be(fragment_offset << u8::BITS); // Needs shifted since we only fill on the most significant bytes.
		packet_type = u8::read_from_big_endian(&mut stream)?;
		quantization = u8::read_from_big_endian(&mut stream)?;
		unscaled_width = u8::read_from_big_endian(&mut stream)?;
		unscaled_height = u8::read_from_big_endian(&mut stream)?;

		width = (unscaled_width as u16) * 8;
		height = (unscaled_height as u16) * 8;

		if packet_type >= 64
		{
			restart_header = Some(JpegRestartHeader::from_stream(&mut stream)?);
		}
		if quantization >= 128 && !ignore_quant
		{
			quantization_header = Some(JQTH::from_stream(&mut stream)?);
		}

		Ok(Self {
			type_specific,
			fragment_offset,
			packet_type,
			quantization,
			width,
			height,
			restart_header,
			quantization_header,
		})
	}
}


#[derive(Debug)]
pub struct JpegRestartHeader
{
	pub restart_interval	: u16,
	pub f					: bool,
	pub l					: bool,
	pub restart_count		: u16,
}

impl JpegRestartHeader
{
	pub fn from_stream(mut stream : impl Read) -> Result<Self, Error>
	{
		const F_BIT : u16 = 0b1000_0000__0000_0000_u16.to_be();
		const L_BIT : u16 = 0b0100_0000__0000_0000_u16.to_be();
		const R_COUNT_MASK : u16 = !(F_BIT | L_BIT);

		let mut restart_interval	: u16 = 0;
		let mut restart_count		: u16 = 0;

		stream.read(restart_interval.as_mut_bytes())?;
		stream.read(restart_count.as_mut_bytes())?;
		restart_interval = u16::from_be(restart_interval);

		let f = (restart_count & F_BIT) != 0;
		let l = (restart_count & L_BIT) != 0;
		restart_count = restart_count & R_COUNT_MASK;


		Ok(Self {
			restart_interval,
			f,
			l,
			restart_count,
		})
	}
}

pub type JQTH = JpegQuantizationTableHeader;
#[derive(Debug)]
pub struct JpegQuantizationTableHeader
{
	pub mbz			: u8,
	pub precision	: u8,
	pub table		: Vec<u8>,
}

impl JpegQuantizationTableHeader
{
	pub fn from_stream(mut stream : impl Read) -> Result<Self, Error>
	{
		let mut mbz: u8 = 0;
		let mut precision : u8 = 0;
		let mut length: u16 = 0;

		stream.read(&mut mbz.as_mut_bytes())?;
		stream.read(&mut precision.as_mut_bytes())?;
		stream.read(&mut length.as_mut_bytes())?;

		length = u16::from_be(length);
		let mut table = vec![0;length as usize];
		stream.read_exact(&mut table)?;


		Ok(Self { mbz, precision, table })
	}
}