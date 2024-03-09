use std::{
    io::{BufRead, BufReader, Write, Lines},
    net::{TcpListener, TcpStream},
};

struct Request {
    path: String,
    headers: Vec<(String, String)>,
}

impl Request {
    fn new(path: String, headers: Vec<(String, String)>) -> Self {
        Request { path, headers }
    }
}

fn parse_request(lines: &mut Lines<BufReader<&TcpStream>>) -> Request {
    let mut headers = Vec::new();
    let mut path = String::new();

    for (i, line) in lines.enumerate() {
        let line = line.unwrap();
        if i == 0 {
            let tokens: Vec<_> = line.split(" ").collect();
            path = tokens[1].to_string();
        } else if !line.is_empty() {
            let header_tokens: Vec<_> = line.split(": ").collect();
            if header_tokens.len() == 2 {
                headers.push((header_tokens[0].to_string(), header_tokens[1].to_string()));
            }
        } else {
            break;
        }
    }

    return Request::new(path, headers)
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
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();
    let request = parse_request(&mut lines);
    
    match request.path {
        _path if _path.starts_with("/echo") => {
            let body_content = _path.strip_prefix("/echo/").unwrap_or(_path.as_str());
            let content_length = format!("Content-Length: {}", &body_content.len());
            let headers = vec!["Content-Type: text/plain", &content_length.as_str()];
            respond(&mut stream, "200 OK", &headers, &body_content);
        }
        _path if _path.starts_with("/user-agent") => {
            let body_content = &request.headers.iter().find(|(k, _)| k == "User-Agent").unwrap().1;
            let content_length = format!("Content-Length: {}", &body_content.len());
            let headers = vec!["Content-Type: text/plain", &content_length.as_str()];
            respond(&mut stream, "200 OK", &headers, &body_content.as_str())
        }
        _path if _path == "/" => {
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

