use mio::net::UdpSocket;
use crate::error::Error;

enum VideoPacketInfo<'a>
{
	PartialIDR(&'a [u8]),
	PPS(&'a [u8]),	// 13 bytes
	SPS(&'a [u8]),	// 13 bytes
	IntermediateFrame(&'a [u8]),
}

pub struct VideoPacket<'a> {
	frame_number	: u8,
	local_number	: u8,
	payload			: VideoPacketInfo<'a>,
}

impl<'a> VideoPacket<'a> {
	const END_OF_FRAME_BITMASK	: u8 =  0b1000_0000;
	const START_OF_PAYLOAD		: usize = 2;

	pub fn frame_number(&self) -> u8 {
		self.frame_number
	}
	pub fn local_number(&self) -> u8 {
		self.local_number
	}
	pub fn payload(&self) -> &'a [u8] {
		match &self.payload {
			VideoPacketInfo::PartialIDR(payload) => { payload }
			VideoPacketInfo::PPS(payload) => { payload }
			VideoPacketInfo::SPS(payload) => { payload }
			VideoPacketInfo::IntermediateFrame(payload) => { payload }
		}
	}
	pub fn is_idr(&self) -> bool { if let VideoPacketInfo::PartialIDR(_) = self.payload { true } else { false } }
	pub fn is_pps(&self) -> bool { if let VideoPacketInfo::PPS(_) = self.payload { true } else { false } }
	pub fn is_sps(&self) -> bool { if let VideoPacketInfo::SPS(_) = self.payload { true } else { false } }
	pub fn is_end_of_frame(&self) -> bool { self.frame_number & Self::END_OF_FRAME_BITMASK != 0 }

	pub fn recv_packet(sock : &mut UdpSocket, frame_buffer : &'a mut [u8]) -> Result<Self, Error> {
		let bytes_read = sock.recv(frame_buffer)?;

		let frame_number = frame_buffer[0];
		let local_number = frame_buffer[1];
		let payload = Self::interpret_payload(&frame_buffer[Self::START_OF_PAYLOAD..bytes_read], frame_number, bytes_read);

		Ok(
			Self {
				frame_number,
				local_number,
				payload,
			}
		)
	} // recv_packet

	fn interpret_payload(payload : &[u8], frame_number : u8, bytes_read : usize) -> VideoPacketInfo {
		// FIXME: These constants, when this works
		if frame_number == 0x80 {
			if bytes_read == 10 { // 8 bytes, mind
				VideoPacketInfo::PPS(payload)
			} else if bytes_read == 15 { // 13 bytes, mind
				VideoPacketInfo::SPS(payload)
			} else {
				panic!("WHAT, THERE'S ANOTHER THING THAT CAN BE 0x80??? {}", payload.len())
			}
		}
		else {
			// NAL unit is 0x65, if we're starting an IDR
			if payload[4] == 0x65 {
				VideoPacketInfo::PartialIDR(payload)
			}
			// We have exhaustively proven that this is just a normal intermediate frame.
			else {
				VideoPacketInfo::IntermediateFrame(payload)
			}
		}

	} // interpret_payload
}
