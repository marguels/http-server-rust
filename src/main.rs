use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    thread
};
use clap::{arg, Parser};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    directory: Option<String>,
}

struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>
}

impl Request {
    fn new(method: String, path: String, headers: Vec<(String, String)>, body: Vec<u8>) -> Self {
        Request { method, path, headers, body }
    }
}

fn parse_request(reader: &mut BufReader<&TcpStream>) -> Request {
    let mut method = String::new();
    let mut headers = Vec::new();
    let mut path = String::new();
    let mut body = Vec::new();

    let mut lines = reader.by_ref().lines();

    if let Some(Ok(first_line)) = lines.next() {
        let tokens: Vec<_> = first_line.split_whitespace().collect();
        if tokens.len() >= 2 {
            method = tokens[0].to_string();
            path = tokens[1].to_string();
        }
    }

    let mut content_length = 0;

    for line in lines {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        if let Some(pos) = line.find(":") {
            let (key, value) = line.split_at(pos);
            let value = value.trim_start_matches(": ");
            headers.push((key.to_string(), value.to_string()));

            if key.to_lowercase() == "content-length" {
                content_length = value.parse::<usize>().unwrap_or(0);
            }
        }
    }

    if content_length > 0 {
        let mut buffer = vec![0; content_length];
        reader.read_exact(&mut buffer).unwrap();
        body = buffer;
    }

    return Request::new(method, path, headers, body);
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

fn handle_request(mut stream: TcpStream, directory: Option<String>) {
    let mut reader = BufReader::new(&stream);
    let request = parse_request(&mut reader);
    
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
        _path if _path.starts_with("/files") => {
            if directory.is_some() {
                match request.method {
                    _method if _method == "GET" => {
                        let filename = _path.strip_prefix("/files/").unwrap();
                        let file_path = Path::new(directory.unwrap().as_str()).join(filename);

                        if file_path.exists() && file_path.is_file() {
                            let file_content = std::fs::read_to_string(file_path).unwrap();
                            let content_length = format!("Content-Length: {}", &file_content.len());
                            let headers: Vec<&str> = vec!["Content-Type: application/octet-stream", &content_length.as_str()];
                            respond(&mut stream, "200 OK", &headers, &file_content);
                        } else {
                            respond(&mut stream, "404 Not Found", &vec![], "");
                        }
                    }
                    _method if _method == "POST" => {
                        let filename = _path.strip_prefix("/files/").unwrap();
                        let file_path = Path::new(directory.unwrap().as_str()).join(filename);

                        let content_length = &request.headers.iter().find(|(k, _)| k == "Content-Length").unwrap().1.parse::<usize>().unwrap();
                        print!("Content-Length: {}", content_length);

                        // let content: String = String::from_utf8(request.body).unwrap();
                        let mut file = fs::File::options().create(true).write(true).open(file_path).unwrap();
                        file.write_all(&request.body).unwrap();
                        file.flush().unwrap();

                        let headers: Vec<&str> = vec!["Content-Type: text/plain"];
                        respond(&mut stream, "201 OK", &headers, "");
                    }
                    _ => {
                        respond(&mut stream, "405 Method Not Allowed", &vec![], "");
                    }
                }
            } else {
                respond(&mut stream, "404 Not Found", &vec![], "");
            }
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
    let args = Args::parse();
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Accepted new connection");
                let directory = args.directory.clone();
                thread::spawn(move || {
                    handle_request(stream, directory);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

