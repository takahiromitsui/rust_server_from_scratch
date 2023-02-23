use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let address = "127.0.0.1:5000";
    // 127.0.0.1 is the local host
    // bind is equivalent to new
    let listener = TcpListener::bind(address).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
        println!("Connection established!");
    }
}

fn handle_get_request(mut stream: TcpStream, is_success: bool, path: &str) {
    let status_line = if is_success {
        "HTTP/1.1 200 OK"
    } else {
        "HTTP/1.1 404 NOT FOUND"
    };
    let contents = fs::read_to_string(path).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    if request_line == "GET / HTTP/1.1" {
        handle_get_request(stream, true, "hello.html")
    } else {
        handle_get_request(stream, false, "404.html")
    }
}
