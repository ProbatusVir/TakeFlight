// https://datatracker.ietf.org/doc/html/rfc2435

use crate::video::rtp::JpegMainHeader;
use crate::error::Error;
use crate::video::rfc2435::Markers::{DriHeaderMarker, EndOfImageMarker, HuffmanTableMarker, QuantizationTableMarker, StartOfFrameMarker, StartOfImageMarker, StartOfScanMarker};
use std::io::Write;
use zerocopy::IntoBytes;

// Luminance constants
// Yes, this is like Directional Current. This makes sense in the context of Discrete Cosine Transform, apparently.
const LUM_DC_CODELENS : [u8;16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const LUM_DC_SYMBOLS : [u8;12] = [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, ];

const LUM_AC_CODELENS : [u8;16] = [ 0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d, ];

const LUM_AC_SYMBOLS : [u8;162] = {
	[0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12,
	0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
	0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08,
	0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0,
	0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16,
	0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28,
	0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
	0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
	0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
	0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
	0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
	0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
	0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98,
	0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
	0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6,
	0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5,
	0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4,
	0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
	0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea,
	0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
	0xf9, 0xfa,] };


// Chrominance constants
const CHM_DC_CODELENS : [u8;16] = [ 0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, ];
const CHM_DC_SYMBOLS : [u8;12] = [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, ];

const CHM_AC_CODELENS : [u8;16] = [ 0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77 ];
const CHM_AC_SYMBOLS : [u8;162] = { [
	0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21,
	0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
	0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91,
	0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0,
	0x15, 0x62, 0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34,
	0xe1, 0x25, 0xf1, 0x17, 0x18, 0x19, 0x1a, 0x26,
	0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38,
	0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
	0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
	0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
	0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78,
	0x79, 0x7a, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
	0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96,
	0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5,
	0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4,
	0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3,
	0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2,
	0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda,
	0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9,
	0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
	0xf9, 0xfa, ]};

const JPEG_LUMA_QUANTIZER: [i32;64] = {[
	16, 11, 10, 16, 24, 40, 51, 61,
	12, 12, 14, 19, 26, 58, 60, 55,
	14, 13, 16, 24, 40, 57, 69, 56,
	14, 17, 22, 29, 51, 87, 80, 62,
	18, 22, 37, 56, 68, 109, 103, 77,
	24, 35, 55, 64, 81, 104, 113, 92,
	49, 64, 78, 87, 103, 121, 120, 101,
	72, 92, 95, 98, 112, 100, 103, 99,
]};
const JPEG_CHROMA_QUANTIZER: [i32;64] = {[
	17, 18, 24, 47, 99, 99, 99, 99,
	18, 21, 26, 66, 99, 99, 99, 99,
	24, 26, 56, 99, 99, 99, 99, 99,
	47, 66, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99
]};

#[repr(u16)]
enum Markers
{
	QuantizationTableMarker	= 0xFFDB_u16.to_be(),
	HuffmanTableMarker		= 0xFFC4_u16.to_be(),
	DriHeaderMarker			= 0xFFDD_u16.to_be(),

	StartOfImageMarker		= 0xFFD8_u16.to_be(),
	StartOfFrameMarker		= 0xFFC0_u16.to_be(),
	StartOfScanMarker		= 0xFFDA_u16.to_be(),

	EndOfImageMarker		= 0xFFD9_u16.to_be(),
	JfifApp0Marker			= 0xFFE0_u16.to_be(),
}

pub struct Thumbnail<'a> 	// probably missing a whole 6 bytes on alignment or something.
{
	image_data	: &'a [u8],
	width		: u8,
	height		: u8,
}

type QuantizationHeader = [u8;132];

/// I am assuming that the table number should be a byte, since the alternative (i32) makes no sense
///
/// qt should be either the luminance or chrominance quant table
fn make_quant_header(p : &mut Vec<u8>, qt : &[u8], table_number : u8) -> Result<(), Error>
{
	debug_assert_eq!(qt.len(), 64);


	p.write_all([
		QuantizationTableMarker as u16,
		0x0043_u16.to_be(), // Length of this table & header (minus the marker) 67_u16
	].as_bytes())?;
	p.write_all(&[table_number])?;

	// Write data
	p.write_all(qt)?;

	Ok(())
}

fn make_huffman_header(p : &mut Vec<u8>, codelens : &[u8], symbols : &[u8],
					   table_number : u8, table_class : u8) -> Result<(), Error>
{
	debug_assert_eq!(codelens.len(), 16);

	p.write_all([
		HuffmanTableMarker as u16,
		((3 + codelens.len() + symbols.len()) as u16).to_be()
	].as_bytes())?;
	p.write_all(&[(table_class << 4) | table_number])?;

	// Write data
	p.write_all(codelens)?;
	p.write_all(symbols)?;

	Ok(())
}

/// Initializes luminance and chrominance tables given a q value.
///
/// lqt and cqt are merely containers, their values at the time they are passed in do not matter
fn make_tables(q : i32, lqt : &mut [u8;64], cqt : &mut [u8;64])
{
	// Apparently, if q is greater than 128, we should use the tables as-is
	if q >= 128 { return }

	let mut factor = q;

	// re-set factor, perhaps
	if q < 1 { factor = 1 } else if q > 99 { factor = 99 }
	// re-set q
	let q = if q < 50 { 5000 / factor } else { 200 - factor * 2 };

	for i in 0..64
	{
		let lq = ((JPEG_LUMA_QUANTIZER[i] * q + 50) / 100).clamp(1, 255);
		let cq = ((JPEG_CHROMA_QUANTIZER[i] * q + 50) / 100).clamp(1, 255);

		lqt[i] = lq as u8;
		cqt[i] = cq as u8;
	}
}

fn make_dri_header(p : &mut Vec<u8>, dri : u16) -> Result<(), Error>
{
	p.write_all([
		DriHeaderMarker as u16,
		0x0004_u16.to_be(),
		dri.to_be(),
	].as_bytes())?;

	Ok(())
}

/// Generate a frame and scan headers that can be prepended to the RTP/JPEG data payload
/// to produce a JPEG compressed image in interchange format (except for possible trailing
/// garbage and absence of EOI marker to terminate the scan).
///
/// ### Arguments:
/// ```
/// * headers_type, width, height: as supplied in RTP/JPEG header
/// * lqt, cqt: quantization tables as either derived from the Q field using make_tables or as specified in section 4.2.
/// * dri: restart interval in MCUs, or 0 if no restarts.
/// * p: pointer to the buffer, we will append the payload to the end of the buffer.
/// ```
pub fn make_headers(p : &mut Vec<u8>, headers_type : u8, blocks_w : u16, blocks_h : u16,
					lqt : &mut [u8], cqt : &mut[u8], dri : u16) -> Result<(), Error>
{
	debug_assert!(headers_type == 0 || headers_type == 1, "For headers_type not 0, or 1, there is no definition in this standard.");

	// convert from blocks to pixels
	let h = blocks_h * 8;
	let w = blocks_w * 8;

	p.write_all((StartOfImageMarker as u16).as_bytes())?;

	make_quant_header(p, lqt, 0)?;
	make_quant_header(p, cqt, 1)?;

	if dri != 0
	{
		make_dri_header(p, dri)?;
	}

	/* Write the start of file and the length of this segment */ {
		p.write_all([
			StartOfFrameMarker as u16,
			17_u16.to_be(), // length of StartOfFile Header (minus the marker)
		].as_bytes())?;

		p.write_all(&[0x08])?; // 8 is for 8-bits of precision (for the pixel components, YUV8, I think)

		// Write out the height in pixels
		p.write_all([
			h.to_be(),
			w.to_be(),
		].as_bytes())?;

		p.write_all(&[
			0x03, // number of components
			0x00, // comp 0
		])?;

		// hsamp = 2;	vsamp = 1 | 2
		p.write_all(&[ if headers_type == 0 { 0x21 } else { 0x22 }])?;

		p.write_all(&[
			0x00,	// quant table 0
			0x01,	// comp 1
			0x11,	// hsamp = 1, vsamp = 1. Looks like sampling quantity is specified per nibble.
			0x01,	// quant table 1
			0x02,	// comp 2
			0x11,	// hsamp = 1, vsamp = 1. Looks like sampling quantity is specified per nibble.
			0x01,	// quant table 1
		])?;

		make_huffman_header(p, &LUM_DC_CODELENS, &LUM_DC_SYMBOLS, 0, 0)?;
		make_huffman_header(p, &LUM_AC_CODELENS, &LUM_AC_SYMBOLS, 0, 1)?;

		make_huffman_header(p, &CHM_DC_CODELENS, &CHM_DC_SYMBOLS, 1, 0)?;
		make_huffman_header(p, &CHM_AC_CODELENS, &CHM_AC_SYMBOLS, 1, 1)?;

	}

	/* Write the start of scan */ {
		p.write_all([
			StartOfScanMarker as u16,
			12_u16.to_be(),	// Length of the start of scan header
		].as_bytes())?;


		p.write_all(&[
			0x03,	// number of components
			0x00,	// comp (y)
			0x00,	// huffman table 0
			0x01,	// comp (cr)
			0x11,	// huffman table 1
			0x02,	// comp (cb)
			0x11,	// huffman table 1
			0x00,	// first DCT coefficient
			63,		// last DCT coefficient
			0x00,	// successive approx.
		])?;
	}

	Ok(())
}

/// all_payloads: The payload for each one of the images.
pub fn create_image(out_buffer : &mut Vec<u8>, jpeg_main_header : &JpegMainHeader, all_payloads : &mut [u8],
					lqt : &mut [u8;64], cqt : &mut [u8;64], _thumbnail : Option<Thumbnail>) -> Result<(), Error>
{
	if jpeg_main_header.packet_type > 127 { return Err(Error::Custom("JPEG packet types above 127 are not implemented."))? }
	let packet_type = jpeg_main_header.packet_type % 64;

	make_tables(jpeg_main_header.quantization as i32, lqt, cqt);

	//out_buffer.write_all((StartOfImageMarker as u16).as_bytes())?;


	/* insert JFIF APP0 */ /* {
		const SIZE_OF_SEGMENT : u16 = 2 + 5 + 2 + 1 + 2 + 2 + 1 + 1;
		let (total_segment_size, thumbnail_w, thumbnail_h) = if thumbnail.is_some() {
				let thumbnail = thumbnail.as_ref().unwrap();
				(SIZE_OF_SEGMENT + thumbnail.image_data.len() as u16, thumbnail.width, thumbnail.height)
			} else {
				(SIZE_OF_SEGMENT, 0, 0)
			};

		out_buffer.write_all([
			JfifApp0Marker as u16,
			total_segment_size.to_be(),
		].as_bytes())?;

		out_buffer.write_all(b"JFIF\0")?;

		out_buffer.write_all(&[
			0x01,	// Major version of JFIF
			0x02,	// Minor version of JFIF
			0x00,	// Units for pixel density. none are specified for this standard.
		])?;

		// TODO: make these 1 again
		out_buffer.write_all([
			2_16,	// horizontal pixel density
			2_u16,	// vertical pixel density
		].as_bytes())?;

		out_buffer.write_all(&[
			thumbnail_w,	// thumbnail width
			thumbnail_h,	// thumbnail height
		])?;

		if thumbnail.is_some() { out_buffer.write_all(thumbnail.unwrap().image_data)?; }
	} */


	make_headers(out_buffer, packet_type,
				 jpeg_main_header.width / 8, jpeg_main_header.height / 8,
				 lqt, cqt,
				 jpeg_main_header.restart_header.as_ref().unwrap().restart_interval)?;

	// FIXME: Presumably, the issue is in the payloads

	out_buffer.write(all_payloads)?;
	out_buffer.write(&(EndOfImageMarker as u16).as_bytes())?;

	Ok(())
}



/***********************************************************************************************
 * This next section is for sending a jpeg in the format of this specification
***********************************************************************************************/

/*
fn send_frame<T : Write>(mut start_seq : u16, ts : u32, ssrc: u32,
			  jpeg_data : &[u8], headers_type : u8,
			  type_spec : u8, width : usize, height : usize, dri : u16,
			  q : u8, lqt : &[u8], ctq : &[u8], packet_size : usize,
			  out_stream : T)
{
	let mut packet_buffer = Vec::with_capacity(packet_size);
	RTPHeader {
		version: 2,
		payload_type : 0,
		sequence_number : 0,
		timestamp : 0,
		ssrc,
		is_last_in_frame: false,
	}
}
*/