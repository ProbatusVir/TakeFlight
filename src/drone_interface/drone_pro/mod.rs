use crate::debug_utils::view_raw_hex;
use crate::error::Error;
use local_ip_address::local_ip;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::str::FromStr;
use std::thread;
use std::time::Duration;

pub mod camera;

//#[test]
pub(crate) fn test() -> Result<(), Error>
{
	/*

	(udp && !dns && !icmp && (ip.addr == 192.168.1.1 || ip.addr == 192.168.169.1)) || tcp.port == 7070
	*/

	let local_ip = local_ip()?;

	// we can make arbitrary port numbers 0
	let sock = UdpSocket::bind(SocketAddr::new(local_ip, 0))?;
	sock.connect("192.168.1.1:7099")?;
	sock.send(&[0x01, 0x01])?;


	// heartbeat
	{
		let other_sock = UdpSocket::bind(SocketAddr::new(local_ip, 0))?;
		other_sock.connect("192.168.169.1:8800")?;
		thread::spawn(move || loop {
			other_sock.send(&[0xef, 0x00, 0x04, 0x00]).unwrap();
			thread::sleep(Duration::from_secs_f32(0.5));
		});
	}


	let mut tcp_socket = TcpStream::connect(SocketAddr::new("192.168.1.1".parse().unwrap(), 7070))?;
	tcp_socket.write(b"OPTIONS rtsp://192.168.1.1:7070/webcam RTSP/1.0\x0d\x0aCSeq: 1\x0d\x0aUser-Agent: Lavf57.71.100\x0d\x0a\x0d\x0a")?;
	dbg!("Sent TCP!");

	let mut tcp_input_buffer = vec![0;256];
	tcp_socket.read(&mut tcp_input_buffer)?;
	//println!("{}", String::from_utf8_lossy(&tcp_input_buffer));

	// Write packet 2
	tcp_socket.write(&[0x44, 0x45, 0x53, 0x43, 0x52, 0x49, 0x42, 0x45, 0x20, 0x72, 0x74, 0x73, 0x70, 0x3a, 0x2f, 0x2f, 0x31, 0x39, 0x32, 0x2e, 0x31, 0x36, 0x38, 0x2e, 0x31, 0x2e, 0x31, 0x3a, 0x37, 0x30, 0x37, 0x30, 0x2f, 0x77, 0x65, 0x62, 0x63, 0x61, 0x6d, 0x20, 0x52, 0x54, 0x53, 0x50, 0x2f, 0x31, 0x2e, 0x30, 0xd, 0xa, 0x41, 0x63, 0x63, 0x65, 0x70, 0x74, 0x3a, 0x20, 0x61, 0x70, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x2f, 0x73, 0x64, 0x70, 0xd, 0xa, 0x43, 0x53, 0x65, 0x71, 0x3a, 0x20, 0x32, 0xd, 0xa, 0x55, 0x73, 0x65, 0x72, 0x2d, 0x41, 0x67, 0x65, 0x6e, 0x74, 0x3a, 0x20, 0x4c, 0x61, 0x76, 0x66, 0x35, 0x37, 0x2e, 0x37, 0x31, 0x2e, 0x31, 0x30, 0x30, 0xd, 0xa, 0xd, 0xa])?;
	tcp_socket.read(&mut tcp_input_buffer)?;
	println!("{}", String::from_utf8_lossy(&tcp_input_buffer));



	// Write packet 3
	tcp_socket.write(&[0x53, 0x45, 0x54, 0x55, 0x50, 0x20, 0x72, 0x74, 0x73, 0x70, 0x3a, 0x2f, 0x2f, 0x31, 0x39, 0x32, 0x2e, 0x31, 0x36, 0x38, 0x2e, 0x31, 0x2e, 0x31, 0x3a, 0x37, 0x30, 0x37, 0x30, 0x2f, 0x77, 0x65, 0x62, 0x63, 0x61, 0x6d, 0x2f, 0x74, 0x72, 0x61, 0x63, 0x6b, 0x30, 0x20, 0x52, 0x54, 0x53, 0x50, 0x2f, 0x31, 0x2e, 0x30, 0xd, 0xa, 0x54, 0x72, 0x61, 0x6e, 0x73, 0x70, 0x6f, 0x72, 0x74, 0x3a, 0x20, 0x52, 0x54, 0x50, 0x2f, 0x41, 0x56, 0x50, 0x2f, 0x55, 0x44, 0x50, 0x3b, 0x75, 0x6e, 0x69, 0x63, 0x61, 0x73, 0x74, 0x3b, 0x63, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x5f, 0x70, 0x6f, 0x72, 0x74, 0x3d, 0x33, 0x30, 0x37, 0x33, 0x32, 0x2d, 0x33, 0x30, 0x37, 0x33, 0x33, 0xd, 0xa, 0x43, 0x53, 0x65, 0x71, 0x3a, 0x20, 0x33, 0xd, 0xa, 0x55, 0x73, 0x65, 0x72, 0x2d, 0x41, 0x67, 0x65, 0x6e, 0x74, 0x3a, 0x20, 0x4c, 0x61, 0x76, 0x66, 0x35, 0x37, 0x2e, 0x37, 0x31, 0x2e, 0x31, 0x30, 0x30, 0xd, 0xa, 0xd, 0xa])?;
	tcp_socket.read(&mut tcp_input_buffer)?;

	let session_id = {
		let session_description = String::from_utf8_lossy(&tcp_input_buffer);
		let mut session_id = String::new();
		for line in session_description.lines()
		{
			let result = line.split_once("Session: ");
			if result.is_some() { session_id = String::from_str(result.unwrap().1)? }
		}
		session_id
	};

	// \r\n

	// Write packet 4
	tcp_socket.write(format!("PLAY rtsp://192.168.1.1:7070/webcam/ RTSP/1.0\r\nRange: npt=0.000-\r\nCSeq: 4\r\nUser-Agent: Lavf57.71.100\r\nSession: {session_id}\r\n\r\n")
		.as_bytes())?;
	tcp_socket.read(&mut tcp_input_buffer)?;

	// video sock
	//let video_sock = UdpSocket::bind(SocketAddr::new(local_ip, 65402))?;
	let video_sock = UdpSocket::bind(SocketAddr::new(local_ip, 0))?; // this number matters since the drone initiates
	video_sock.connect("192.168.1.1:52612")?;
	video_sock.send(&[0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0])?;


	let video_stream_sock = UdpSocket::bind(SocketAddr::new(local_ip, 30732))?;

	let mut frame_buf = vec![0; 4096];
	let bytes_read = video_stream_sock.recv(&mut frame_buf)?;
	let mut frame_cursor = Cursor::new(&frame_buf[0..bytes_read]); // This slice doesn't actually *really* matter... just makes some things a little nicer.
	let _header = camera::RTPHeader::from_stream(&mut frame_cursor)?;
	let _byte_after_header = frame_cursor.position();
	let jpeg_header = camera::JpegMainHeader::from_stream(&mut frame_cursor, false)?;

	// Create frames and copy the first buffer's data
	let mut frame = Vec::new();
	//frame.extend_from_slice(&frame_buf[(byte_after_header) as usize..bytes_read]); // Read only the payload information from the packet -- this should be part of the other code
	frame.extend_from_slice(&frame_buf[frame_cursor.position() as usize..bytes_read]); // Read only the payload information from the packet -- this should be part of the other code
	let mut lqt : [u8;64] = [0;64];
	let mut cqt: [u8;64] = [0;64];

	// initialize the dang tables >:(
	{
		let jpeg_information = jpeg_header.quantization_header.as_ref().unwrap();
		lqt.clone_from_slice(&jpeg_information.table[..64]);
		cqt.clone_from_slice(&jpeg_information.table[64..]);
	}
	let mut out_buffer = Vec::new();
	let mut number_of_packets = 1;

	println!("Number of packets: {number_of_packets}");
	loop
	{
		let bytes_read = video_stream_sock.recv(&mut frame_buf)?;
		frame_cursor = Cursor::new(&frame_buf);
		// Strip the RTP header from the stream
		let new_header = camera::RTPHeader::from_stream(&mut frame_cursor)?;
		let _jpeg_header = camera::JpegMainHeader::from_stream(&mut frame_cursor, true)?;

		// we're gonna assume that the images are all sent in order.
		{
			// Add only the jpeg data. Markers and payload
			frame.extend_from_slice(&frame_buf[frame_cursor.position() as usize..bytes_read]);
			println!("We've received {number_of_packets} packets!");
			number_of_packets += 1;
		}
		if
		new_header.is_last_in_frame {
			crate::video::rfc2435::create_image(&mut out_buffer, &jpeg_header, &mut frame, &mut lqt, &mut cqt, None)?;
			break /* TODO: add logic here */
		}
	}
	//File::create("test_results/one_frame.raw")?.write_all(&frame)?;
	File::create("test_results/decoded_picture.jpeg")?.write_all(&out_buffer)?;
	Ok(())
}