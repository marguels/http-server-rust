use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

struct Request {
    method: String,
    path: String,
}

impl Request {
    fn new(method: String, path: String) -> Self {
        Request { method, path }
    }
}

fn parse_request(first_line: String) -> Request {
    let first_line_tokens: Vec<_> = first_line.split(" ").collect();
    return Request::new(
        String::from(first_line_tokens[0]),
        String::from(first_line_tokens[1]),
    );
}

fn handle_request(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();

    let _ = reader.read_line(&mut line);
    let request = parse_request(line);
    if request.path == "/" {
        stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        stream.flush().unwrap();
    } else {
        stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
        stream.flush().unwrap();
    }
}

fn main() {

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted new connection");
                handle_request(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

