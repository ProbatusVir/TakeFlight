//Basic HTTP server implementation
use std::{
    io::{prelude::*,BufReader },
    net::{TcpStream, TcpListener, UdpSocket},
};

fn server(){
    //bind to localhost:5137
    let listener = TcpListener::bind("127.0.0.1:5137").unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    //Accept incoming TCP connections
    for stream in listener.incoming(){
        let stream = stream.unwrap();
        //Handle each connection
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream){
    //Initialize buffer
    let mut buffer = [0; 512];
    //Read the HTTP request into the buffer
    stream.read(&mut buffer).unwrap();
    //For Debugging: Converts bytes to string
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    //Create Simple HTTP response
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    //Send it to client
    stream.write(response.as_bytes()).unwrap();
    //Ensures anything using write/write_all is sent out
    stream.flush().unwrap();
}
