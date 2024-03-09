use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

struct Request {
    path: String,
}

impl Request {
    fn new(path: String) -> Self {
        Request { path }
    }
}

fn parse_request(first_line: String) -> Request {
    let first_line_tokens: Vec<_> = first_line.split(" ").collect();
    return Request::new(
        String::from(first_line_tokens[1]),
    );
}

fn respond(stream: &mut TcpStream, status: &str, headers: &Vec<&str>, body: &str) {
    let string_headers = headers.join("\r\n");
    let response = format!("HTTP/1.1 {}\r\n{}\r\n\r\n{}",
        status,
        string_headers,
        body);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_request(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();

    let _ = reader.read_line(&mut line);
    let request = parse_request(line);
    
    match request.path {
        _path if _path.starts_with("/echo") => {
            let body_content = _path.strip_prefix("/echo/").unwrap_or(_path.as_str());
            let content_length = format!("Content-Length: {}", &body_content.len());
            let headers = vec!["Content-Type: text/plain", &content_length.as_str()];
            respond(&mut stream, "200 OK", &headers, &body_content);
        }
        path if path == "/" => {
            respond(&mut stream, "200 OK", &vec![], "")
        }
        _ => {
            respond(&mut stream, "404 Not Found", &vec![], "")
        }
    }
}

fn main() {

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Accepted new connection");
                handle_request(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

