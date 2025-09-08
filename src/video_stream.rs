use local_ip_address::local_ip;
use std::io::Error;
use std::net::{ Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use openh264::decoder::Decoder;

#[allow(dead_code)]
/// This is for an incoming video stream
pub struct VideoStream {
	stream: UdpSocket,
	decoder: Decoder
}

#[allow(dead_code)]
impl VideoStream {
	// We might put some parameters on here, maybe that backend enum or something
	// Might get rid of local_port and sub it for literally any random port.
	pub fn new(local_port: u16) -> Result<Self, Error> {
		const VIDEO_PORT: u16 = 1111;
		const VIDEO_STREAM_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 10, 1); //
		const VIDEO_STREAM_SOCKET: SocketAddrV4 =
			SocketAddrV4::new(VIDEO_STREAM_ADDRESS, VIDEO_PORT);

		let local_ip = local_ip()
			.map_err(|_| Error::other("Could not obtain this device's local network IP"))?;
		let local_sock = SocketAddr::new(local_ip, local_port);

		let stream = UdpSocket::bind(local_sock)?;
		let decoder = Decoder::new().map_err(|_| Error::other("Failed to initialize VideoStream decoder"))?;

		Ok(Self { stream, decoder })
	}
}
