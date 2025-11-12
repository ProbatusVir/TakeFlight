/*use crate::error::Error;
use httparse::Status;
use std::io::{Cursor, Read, Write};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct DroneNames {
    names: Vec<String>,
}
//test compatible version of handle_connection in main.rs
fn handle_test<T: Read + Write>(mut stream: T) -> Result<(), Error>{
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

    //GET handle for drone names
    let (status, body) = if req.method == Some("GET")
        && req.path == Some("/drone_names"){
        //populates drone vector with random string names
        let data = crate::DroneNames {
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
        let msg = serde_json::to_string(&crate::DroneNames {
            names: vec!["Invalid request".into()],
        }).unwrap();
        ("HTTP/1.1 404 ERR", msg)
    };
    //Create HTTP response with proper formatting
    let response = format!(
        "{status}\r\nContent-Type: application/json
        \r\nContent-Length: {}\r\n\r\n{body}", body.len()
    );
    //Send it to client
    stream.write(response.as_bytes())?;
    //Ensures anything using write/write_all is sent out
    stream.flush()?;
    Ok(())
}

#[test]
fn http_parse_complete(){
    //Create header
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    //give complete request
    let data = b"GET /drone_names HTTP/1.1\r\nHost: localhost\r\n\r\n";

    let status = req.parse(data);
    //match check
    match status{
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
    };
    assert!(matches!(status, Ok(Status::Complete(_))));
}
#[test]
fn http_parse_partial(){
    //Create header
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    //give partial
    let data = "GET / HTTP/1.1";
    let status = req.parse(data.as_bytes());
    //match check
    match status{
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
    };
    assert!(matches!(status, Ok(Status::Partial)));
}
#[test]
fn http_parse_invalid(){
    //Create header
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    //give complete request
    let data = b"HTTP/0";

    let status = req.parse(data);
    //match check
    match status{
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
    };
    assert!(matches!(status, Err(_)));
}
#[test]
fn test_handle_connection_names(){
    //Simulate GET request
    let request = b"GET /drone_names HTTP/1.1\r\nContent-Type: application/json\
    \r\nHost: localhost\r\n\r\n";

    //Create mock stream
    let mut stream = Cursor::new(Vec::new());
    stream.write_all(request).unwrap();
    stream.set_position(0); // reset to start for reading

    handle_test(&mut stream).unwrap();

    //Extract what was written
    let output = String::from_utf8(stream.get_ref().clone()).unwrap();
    //Verify status
    assert!(output.contains("HTTP/1.1 200 OK"));
    assert!(output.contains("Content-Type: application/json"));

    //Verify JSON body
    let json_start = output.find("{").unwrap();
    let json_end = &output[json_start..output.len()].to_string();
    let parsed: DroneNames = serde_json::from_str(json_end).unwrap();
    assert_eq!(parsed.names, ["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Fern", "Germany"]);
    assert_eq!(parsed.names.len(), 7);
}
#[test]
fn test_handle_connection_invalid(){
    let request = b"GET / HTTP/1.1\r\nContent-Type: application/json\
    \r\nHost: localhost\r\n\r\n";
    //Create mock stream
    let mut stream = Cursor::new(Vec::new());
    stream.write_all(request).unwrap();
    stream.set_position(0); // reset to start for reading

    handle_test(&mut stream).unwrap();

    //Extract what was written
    let output = String::from_utf8(stream.get_ref().clone()).unwrap();
    //Verify 404
    assert!(output.contains("HTTP/1.1 404 ERR"));
}*/