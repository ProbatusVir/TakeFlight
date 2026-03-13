use std::io::Cursor;
use mio::net::UdpSocket;
use crate::error::Error;


// FIXME: we don't need allocations for this
enum VideoPacketInfo
{
	PartialIDR(Box<[u8]>),
	PPS(Box<[u8]>),	// 13 bytes
	SPS(Box<[u8]>),	// 13 bytes
	IntermediateFrame(Box<[u8]>),
}

pub struct VideoPacket {
	frame_number	: u8,
	local_number	: u8,
	payload			: VideoPacketInfo,
}

impl VideoPacket {
	const END_OF_FRAME_BITMASK	: u8 =  0b1000_0000;
	const START_OF_PAYLOAD		: usize = 2;

	pub fn frame_number(&self) -> u8 {
		self.frame_number
	}
	pub fn local_number(&self) -> u8 {
		self.local_number
	}
	pub fn payload(&self) -> &[u8] {
		match &self.payload {
			VideoPacketInfo::PartialIDR(inner) => { &inner }
			VideoPacketInfo::PPS(inner) => { &inner }
			VideoPacketInfo::SPS(inner) => { &inner }
			VideoPacketInfo::IntermediateFrame(inner) => { &inner }
		}
	}
	pub fn is_idr(&self) -> bool { if let VideoPacketInfo::PartialIDR(_) = self.payload { true } else { false } }
	pub fn is_pps(&self) -> bool { if let VideoPacketInfo::PPS(_) = self.payload { true } else { false } }
	pub fn is_sps(&self) -> bool { if let VideoPacketInfo::SPS(_) = self.payload { true } else { false } }
	pub fn is_end_of_frame(&self) -> bool { self.frame_number & Self::END_OF_FRAME_BITMASK != 0 }

	pub fn recv_packet(sock : &mut UdpSocket, read_buf : &mut [u8]) -> Result<Self, Error> {
		let bytes_read = sock.recv(read_buf)?;

		let frame_number = read_buf[0];
		let local_number = read_buf[1];
		let payload = Self::interpret_payload(read_buf[Self::START_OF_PAYLOAD..bytes_read].into(), local_number, bytes_read);

		Ok(
			Self {
				frame_number,
				local_number,
				payload,
			}
		)
	} // recv_packet

	fn interpret_payload(payload : Box<[u8]>, local_number: u8, bytes_read : usize) -> VideoPacketInfo {
		// FIXME: These constants, when this works
		if local_number == 0x80 {
			if bytes_read == 10 { // 8 bytes, mind
				VideoPacketInfo::PPS(payload)
			} else if bytes_read == 15 { // 13 bytes, mind
				VideoPacketInfo::SPS(payload)
			} else {
				VideoPacketInfo::IntermediateFrame(payload)
			}
		} else {
			// NAL unit is 0x65, if we're starting an IDR
			if payload.len() > 4 {
				if payload[4] == 0x65 {
					VideoPacketInfo::PartialIDR(payload)
				} else {
					VideoPacketInfo::IntermediateFrame(payload)
				}
			}
			// We have exhaustively proven that this is just a normal intermediate frame.
			else {
				VideoPacketInfo::IntermediateFrame(payload)
			}
		}

	} // interpret_payload
}
