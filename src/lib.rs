use std::{net::SocketAddr, os::unix::prelude::RawFd};

use std::io;

pub struct MyTcpStream {
    fd: RawFd,
}

impl MyTcpStream {
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n =
            nix::unistd::read(self.fd, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(n)
    }

    pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = nix::unistd::write(self.fd, buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(n)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct MyTcpListener {
    fd: RawFd,
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
        let fd = nix::sys::socket::socket(domain, socket_type, flags, protocol)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // bind the socket to the address
        let addr =
            nix::sys::socket::SockAddr::new_inet(nix::sys::socket::InetAddr::from_std(&addr));
        nix::sys::socket::bind(fd, &addr)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Step3:Wait for incoming connections
        // defines the maximum number of pending connections that can be queued up before connections are refused.
        //e.g., 10
        nix::sys::socket::listen(fd, 10)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        // println!("Listening on {:?}", addr);
        Ok(Self { fd })
    }

    pub fn accept(&self) -> Result<MyTcpStream, std::io::Error> {
            let stream_fd = match nix::sys::socket::accept(self.fd) {
                Ok(fd) => fd,
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                }
            };
            // println!("Accepted connection on fd {}", stream_fd);
            return Ok(MyTcpStream::new(stream_fd));
    }

    pub fn serve_html(buffer:&mut [u8], stream: &mut MyTcpStream, root: &str) -> Result<(), std::io::Error> {
        let val_read = stream.read(buffer);
        let request = String::from_utf8_lossy(&buffer[..val_read.unwrap()]);
        let request_lines: Vec<&str> = request.lines().collect();
        let request_line = request_lines[0];
        let tokens: Vec<&str> = request_line.split_whitespace().collect();

        // get the requested file path from the URL
        let file_path = if tokens[1] == "/" {
            "/index"
        } else {
            tokens[1]
        };
        let file = std::fs::read_to_string(format!("{}{}.html", root, file_path));
        let not_found = std::fs::read_to_string(format!("{}/404.html", root));
        // write back to the new socket
        let response = match file {
            Ok(body) => {
                format!(
                    "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: {}\n\n{}",
                    body.len(),
                    body
                )
            }
            Err(_) => {
                let body = match not_found {
                    Ok(body) => body,
                    Err(_) => "404 Not Found".to_string(),
                };
                format!(
                    "HTTP/1.1 404 Not Found\nContent-Type: text/html\nContent-Length: {}\n\n{}",
                    body.len(),
                    body
                )
            }
        };
        stream.write(response.as_bytes())?;
        Ok(())
    }

}
