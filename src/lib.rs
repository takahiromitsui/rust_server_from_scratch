mod models;

use std::{net::SocketAddr, os::unix::prelude::RawFd};
pub struct MyTcpListener {
    fd: RawFd, // raw file descriptor
}

impl MyTcpListener {
    // step 2: Identify a socket
    pub fn bind(addr: SocketAddr) -> Result<MyTcpListener, std::io::Error> {
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

        // Step3:Wait for incoming connections
        // defines the maximum number of pending connections that can be queued up before connections are refused.
        //e.g., 10
        nix::sys::socket::listen(fd, 10).unwrap();
        Ok(MyTcpListener { fd })
    }
    pub fn accept(&self) {
        loop {
            // accept a new connection
            let new_fd = match nix::sys::socket::accept(self.fd) {
                Ok(fd) => fd,
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                    continue;
                }
            };

            // Step4: Send and receive messages
            // read from the new socket
            let mut buf = [0u8; 3000];
            match nix::unistd::read(new_fd, &mut buf) {
                Ok(val_read) if val_read > 0 => {
                    // write back to the new socket
                    let hello = "HTTP/1.1 200 OK\nContent-Type: text/plain\nContent-Length: 12\n\nHello world!";
                    match nix::unistd::write(new_fd, hello.as_bytes()) {
                        Ok(_) => println!("Sent response: {}", hello),
                        Err(e) => println!("Error sending response: {}", e),
                    }
                }
                Ok(_) => println!("Empty message received"),
                Err(e) => println!("Error reading data: {}", e),
            }

            // Step5: Close the new socket
            match nix::unistd::close(new_fd) {
                Ok(_) => println!("Closed connection"),
                Err(e) => println!("Error closing socket: {}", e),
            }
        }
    }
}
