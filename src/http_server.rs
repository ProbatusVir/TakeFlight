/*use crate::{error, LISTENER};
use error::Error;
use std::{
    collections::HashMap,
    io::ErrorKind,
    net::SocketAddr,
};
use std::io::{Read, Write};
use httparse::{Request, Response, Status};
use serde::{
    Deserialize,
    Serialize
};
//HTTP server implementation
use mio::{
    net::{TcpListener, TcpStream, UdpSocket},
    Events, Interest, Poll, Registry, Token, Waker,
};

#[derive(Serialize, Deserialize)]
struct DroneNames{
    names: Vec<String>
}
struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    response: Vec<u8>,
    closed: bool,
}
const SERVER: Token = Token(0); //Server token
fn server() -> Result<(), Error>{
    //create poll instance
    let mut poll = Poll::new()?;
    //Storage for events
    let mut events = Events::with_capacity(1024);
    //set up the TCP socket
    let addr = "127.0.0.1:5137".parse()?;
    let mut server = TcpListener::bind(addr)?;
    poll.registry()
        .register(&mut server, LISTENER, Interest::READABLE)?;

    //map of 'Token -> TCPStream'
    let mut clients: HashMap<Token, TcpStream> = HashMap::new();
    let mut unique_token = Token(SERVER.0 + 1);

    println!("Listening on {}", server.local_addr()?);

    //event loop
    loop{
        poll.poll(&mut events, None)?;
        for event in events.iter() {
            match event.token() {
                SERVER => {
                    //Accepting new connections
                    match server.accept() {
                        Ok((mut socket, addr)) => {
                            println!("Accepted connection from {}", addr);
                            let token = Token(unique_token.0 + 1);
                            //register clients for readable events
                            poll.registry()
                                .register(&mut socket, token, Interest::READABLE)?;
                            //add connected client to hashmap
                            clients.insert(token, socket);
                        }
                        Err(e) => {
                            if e.kind() == ErrorKind::WouldBlock {continue}
                            else { return Err(e.into()) }
                        }
                    }
                }
                token => {
                    //Client ready to read
                    if let Some(client) = clients.get_mut(&token){
                        if let Err(e) = handle_connection(client)?{
                            eprintln!("Error on {:?}: {}",token, e );
                        }
                    }
                }
            }
        }
    }
}

fn handle_connection(mut stream: &mut TcpStream) -> Result<(), Error>{
    //Initialize buffer
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;
    //create a request object and a buffer for headers
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    //Try parsing the HTTP request
    match req.parse(&buffer[..n]) {
        Ok(Status::Complete(_)) =>{
            println!("Method: {:?}", req.method);
            println!("Path: {:?}", req.path);
            println!("Version: {:?}", req.version);
            println!("Headers: {:?}", req.headers);
        }
        Ok(Status::Partial) => {
            eprintln!("Request is Incomplete");
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
        }
    }
    //Grabs the type of request command
    let request = String::from_utf8_lossy(&buffer[..]);
    //For Debugging: Converts bytes to string
    println!("Request: {}", request);

    //GET handle for drone names
    let (status, body) = if req.method == Some("GET")
        && req.path == Some("/drone_names"){
        //populates drone vector with random string names
        let data = DroneNames{
            names: vec![
                "Alpha".into(),
                "Bravo".into(),
                "Charlie".into(),
                "Delta".into(),
                "Echo".into(),
                "Fern".into(),
                "Germany".into(),
            ],
        };
        //Serializes the vector to JSON String
        let json = serde_json::to_string(&data).unwrap();
        ("HTTP/1.1 200 OK", json)
    }else{
        let msg = serde_json::to_string(&DroneNames{
            names: vec!["Invalid request".into()],
        }).unwrap();
        ("HTTP/1.1 200 OK", msg)
    };
    //Create HTTP response with proper formatting
    let response = format!(
        "{status}\r\nContent-Type: application/json\
        \r\nContent-Length: {}\r\n\r\n{body}", body.len()
    );
    //Send it to client
    stream.write(response.as_bytes())?;
    //Ensures anything using write/write_all is sent out
    stream.flush()?;
    Ok(())
}
*/