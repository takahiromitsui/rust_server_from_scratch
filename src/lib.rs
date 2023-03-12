use std::sync::Mutex;
use std::{collections::HashMap, net::SocketAddr, os::unix::prelude::RawFd, sync::Arc};
pub struct MyTcpListener {
    fd: RawFd, // raw file descriptor
    routes: HashMap<String, String>,
}

impl MyTcpListener {
    // step 2: Identify a socket
    pub fn bind(
        addr: SocketAddr,
        routes: HashMap<String, String>,
    ) -> Result<MyTcpListener, std::io::Error> {
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
        Ok(MyTcpListener { fd, routes })
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
            // clone the routes
            let routes = Arc::new(Mutex::new(self.routes.clone()));

            // spawn a new thread to handle the incoming connection
            std::thread::spawn(move || {
                // Step4: Send and receive messages
                // read from the new socket
                let mut buf = [0u8; 3000];
                match nix::unistd::read(new_fd, &mut buf) {
                    Ok(val_read) if val_read > 0 => {
                        // parse HTTP request
                        let request = std::str::from_utf8(&buf).unwrap();
                        let request_lines: Vec<&str> = request.lines().collect();
                        let request_line = request_lines[0];
                        let tokens: Vec<&str> = request_line.split_whitespace().collect();

                        // get the requested file path from the URL
                        let file_path = if tokens[1] == "/" {
                            "/hello"
                        } else {
                            tokens[1]
                        };
                        // find the corresponding handler for the requested route
                        let handler = routes.lock().unwrap().get(file_path).cloned();

                        // write back to the new socket
                        let response = match handler {
                            Some(f) => {
                                let body = f;
                                format!(
                                    "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: {}\n\n{}",
                                    body.len(),
                                    body
                                )
                            }
                            None => {
                                let body = "404 Not Found";
                                format!(
                                    "HTTP/1.1 404 Not Found\nContent-Type: text/html\nContent-Length: {}\n\n{}",
                                    body.len(),
                                    body
                                )
                            }
                        };
                        match nix::unistd::write(new_fd, response.as_bytes()) {
                            Ok(_) => println!("Sent response: {}", response),
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
            });
        }
    }
}
