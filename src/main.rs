use std::{
    net::{SocketAddr, IpAddr, Ipv4Addr},
    os::unix::prelude::{RawFd},
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
        let addr =
            nix::sys::socket::SockAddr::new_inet(nix::sys::socket::InetAddr::from_std(&addr));
        nix::sys::socket::bind(fd, &addr).unwrap();
        // defines the maximum number of pending connections that can be queued up before connections are refused.
        //e.g., 10
        nix::sys::socket::listen(fd, 10).unwrap();
        Ok(MyTcpListener { fd })
    }
    fn accept(&self) {
        let mut buf = [0u8; 3000];
        // step4: send and receive messages
        match nix::unistd::read(self.fd, &mut buf) {
            Ok(_val_read) => {
                let hello = "Hello from server";
                nix::unistd::write(self.fd, hello.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        // step5: close the socket
        match nix::unistd::close(self.fd) {
            Ok(_) => println!("Successfully closed the socket"),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn main() {
    // let address = "127.0.0.1:5000";
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    // 127.0.0.1 is the local host
    // bind is equivalent to new
    // let listener = TcpListener::bind(address).unwrap();
    let listener = MyTcpListener::bind(socket).unwrap();
    listener.accept();
    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //     handle_connection(stream);
    //     println!("Connection established!");
    // }
}

// fn handle_get_request(mut stream: TcpStream, is_success: bool, path: &str) {
//     let status_line = if is_success {
//         "HTTP/1.1 200 OK"
//     } else {
//         "HTTP/1.1 404 NOT FOUND"
//     };
//     let contents = fs::read_to_string(path).unwrap();
//     let length = contents.len();
//     let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
//     stream.write_all(response.as_bytes()).unwrap();
// }

// fn handle_connection(stream: TcpStream) {
//     let buf_reader = BufReader::new(&stream);
//     let request_line = buf_reader.lines().next().unwrap().unwrap();

//     if request_line == "GET / HTTP/1.1" {
//         handle_get_request(stream, true, "hello.html")
//     } else {
//         handle_get_request(stream, false, "404.html")
//     }
// }
