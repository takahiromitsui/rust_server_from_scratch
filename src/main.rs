use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream, SocketAddr}, os::unix::prelude::{RawFd, AsRawFd}, error::Error,
};

struct MyTcpListener {
    fd: RawFd, // raw file descriptor
}

impl MyTcpListener {
    fn bind(addr: SocketAddr) -> Result<MyTcpListener, std::io::Error> {
        // e.g., Inet = IPv4, Inet6 = IPv6
        let domain = nix::sys::socket::AddressFamily::Inet;
        // e.g., Stream = Provides sequenced, reliable, two-way, connection- based byte streams. An out-of-band data transmission mechanism may be supported.
        let socket_type = nix::sys::socket::SockType::Stream;
        // additional socket options
        let flags = nix::sys::socket::SockFlag::empty();
        // e.g., Tcp = TCP, Udp = UDP
        let protocol = nix::sys::socket::SockProtocol::Tcp;
        // file descriptor
        let fd = nix::sys::socket::socket(domain, socket_type, flags, protocol).unwrap();

        // bind the socket to the address
        let addr = nix::sys::socket::SockAddr::new_inet(nix::sys::socket::InetAddr::from_std(&addr));nix::sys::socket::bind(fd, &addr).unwrap();
        // defines the maximum number of pending connections that can be queued up before connections are refused.
        //e.g., 10
        nix::sys::socket::listen(fd, 10).unwrap();
        Ok(MyTcpListener { fd })
    }
}



fn main() {
    let address = "127.0.0.1:5000";
    // 127.0.0.1 is the local host
    // bind is equivalent to new
    // let listener = TcpListener::bind(address).unwrap();
    let listener = MyTcpListener::bind(address.parse().unwrap()).unwrap();
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
