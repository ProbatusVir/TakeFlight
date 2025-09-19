#[allow(dead_code)]
const HEADER : u8 = 0xCC;

#[allow(dead_code)]
// These values are all three bits
enum PacketTypeValue
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

#[allow(dead_code)]
struct Packet
{
	message : Vec<u8>
}


