use crate::error;
use error::Error;
use serde::{
    Deserialize,
    Serialize
};
//HTTP server implementation
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[derive(Serialize, Deserialize)]
struct DroneNames{
    names: Vec<String>
}

#[tokio::main]
async fn server() -> Result<(), Error>{
    //bind to localhost:5137
    let listener = TcpListener::bind("127.0.0.1:5137").await?;
    println!("Listening on {}", listener.local_addr()?);

    //Accept incoming TCP connections
    loop{
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        tokio::spawn(async move {
            if let Error::IOError(e) = handle_connection(stream).await {
                eprintln!("Error: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), Error>{
    //Initialize buffer
    let mut buffer = [0; 512];
    //Reads the request size/bytes
    if let Ok(size) = stream.read(&mut buffer).await?{
        if size == 0{
            return Ok(());
        }
    }
    //Grabs the type of request command
    let request = String::from_utf8_lossy(&buffer[..]);
    //For Debugging: Converts bytes to string
    println!("Request: {}", request);

    //GET handle for drone names
    let (status, body) = if request.contains("drone_names"){
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
    stream.write(response.as_bytes()).await?;
    //Ensures anything using write/write_all is sent out
    stream.flush().await?;
    Ok(())
}
