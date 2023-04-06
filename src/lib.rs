use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::{net::SocketAddr, os::unix::prelude::RawFd};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io, thread};

#[derive(Serialize, Deserialize)]
pub struct Message {
    text: String,
    guest: String,
}

pub struct MyTcpStream {
    fd: RawFd,
}

impl MyTcpStream {
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
    }
    // nix::unistd::read can be error prone
    // read may return before it has filled the entire buffer if it receives a signal. In that case, you may need to call read again with the remaining buffer to get the rest of the data.
    // read can block indefinitely if there is no data available to be read, which can be a problem in some cases.
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        println!("Reading from stream...{}", buf.len());
        println!("self.fd: {}", self.fd);
        let n =
            nix::unistd::read(self.fd, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        println!("Read {:?} bytes from stream", n);
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

    pub fn get_headers(stream: &mut TcpStream) -> Vec<String> {
        // default buffer is 8KB
        let buf_reader = io::BufReader::new(stream);
        let headers: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();
        headers
    }


    pub fn read_body(stream: &mut TcpStream, content_length: usize) -> String {
        let mut body = String::new();
        let mut buf = vec![0; content_length];
        stream.read_exact(&mut buf).unwrap();
        body.push_str(&String::from_utf8(buf).unwrap());
        println!("body: {}", body);
        body
    }

    pub fn handle_connection(mut stream: TcpStream, root: &str) {
        let headers = MyTcpListener::get_headers(&mut stream);
        let method = headers[0].split_whitespace().nth(0).unwrap();
        let path = headers[0].split_whitespace().nth(1).unwrap();

        if method == "GET" {
            Self::serve_html(stream, root, path);
        } else if method == "POST" {
            Self::post_json(stream, path, Self::update_json);
        } else {
            println!("Method not supported");
        }
    }

    pub fn serve_html(mut stream: TcpStream, root: &str, path: &str) {
        let mut file_path = root.to_string();
        if path == "/" {
            file_path.push_str("/index");
        } else {
            file_path.push_str(path);
        }

        let file = std::fs::read_to_string(format!("{}.html", file_path));
        let not_found = std::fs::read_to_string(format!("{}/404.html", root));
        // // write back to the new socket
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
        stream.write_all(response.as_bytes()).unwrap();

        // let mut buffer = [0; 1024];
        // let val_read = stream.read(&mut buffer);
        // let request = String::from_utf8_lossy(&buffer[..val_read.unwrap()]);
        // let request_lines: Vec<&str> = request.lines().collect();
        // let request_line = request_lines[0];
        // let tokens: Vec<&str> = request_line.split_whitespace().collect();
        // //
        // let headers = request_lines[1..].to_vec();
        // let mut content_length: Option<usize> = None;
        // for header in headers {
        //     let parts: Vec<&str> = header.splitn(2, ": ").collect();
        //     if parts.len() == 2 && parts[0].to_lowercase() == "content-length" {
        //         content_length = Some(parts[1].parse().unwrap());
        //         println!("{:?}", content_length);
        //         break;
        //     }
        // }
        // let content_length = content_length.unwrap_or(0);
        // if content_length> buffer.len() {
        //     println!("Buffer is not enough")
        // }

        // // get the requested file path from the URL
        // let (file_path, is_post) = if tokens[1] == "/" {
        //     ("/index", false)
        // } else {
        //     let mut parts = tokens[1].splitn(2, '?');
        //     let file_path = parts.next().unwrap();
        //     let is_post = parts.next().map_or(false, |p| p == "post=true");
        //     (file_path, is_post)
        // };
        // if tokens[0] == "POST" && tokens[1] == "/message" {
        //     Self::post_json(stream, "/message", Self::update_json);
        //     return Ok(())
        // }

        // let file = std::fs::read_to_string(format!("{}{}.html", root, file_path));
        // let not_found = std::fs::read_to_string(format!("{}/404.html", root));
        // // write back to the new socket
        // let response = match file {
        //     Ok(body) => {
        //         format!(
        //             "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: {}\n\n{}",
        //             body.len(),
        //             body
        //         )
        //     }
        //     Err(_) => {
        //         let body = match not_found {
        //             Ok(body) => body,
        //             Err(_) => "404 Not Found".to_string(),
        //         };
        //         format!(
        //             "HTTP/1.1 404 Not Found\nContent-Type: text/html\nContent-Length: {}\n\n{}",
        //             body.len(),
        //             body
        //         )
        //     }
        // };
        // stream.write(response.as_bytes())?;
        // Ok(())
    }

    pub fn update_json(body: &str) -> Result<Message, String> {
        serde_json::from_str(body).map_err(|e| e.to_string())
    }

    pub fn post_json<F>(mut stream: TcpStream, path: &str, handler: F) {
        println!("post_json is called");

        // let mut buffer = [0; 1024];
        // let mut input = String::new();

        // loop {
        //     match stream.read(&mut buffer) {
        //         Ok(n) if n > 0 => {
        //             input.push_str(std::str::from_utf8(&buffer[..n]).unwrap());
        //             println!("input: {}", input);
        //             if input.contains("\r\n\r\n") {
        //                 break; // End of headers
        //             }
        //         }
        //         Ok(_) => break, // End of stream
        //         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        //             // Wait for more data
        //             thread::sleep(std::time::Duration::from_millis(10));
        //         }
        //         Err(e) => {
        //             eprintln!("Error reading from stream: {}", e);
        //             return;
        //         }
        //     }
        // }

        // println!("input: {}", input);

        // let response = match serde_json::from_str::<Value>(&input) {
        //     Ok(json) => serde_json::to_string(&json).unwrap(),
        //     Err(e) => format!("{{ \"error\": \"{}\" }}", e),
        // };
        // let response = format!(
        //     "HTTP/1.1 200 OK\nContent-Type: application/json\nContent-Length: {}\n\n{}",
        //     response.len(),
        //     response
        // );
        // stream.write(response.as_bytes()).unwrap();
    }

    // where
    //     F: Fn(&str) -> Result<Message, String> + Send + Sync + 'static,
    // {
    //         println!("post_json is called");

    //         let mut buffer = [0; 1024];
    //         let val_read = stream.read(&mut buffer);
    //         if val_read.is_err() {
    //             println!("Error reading from connection: {}", val_read.err().unwrap());
    //             return;
    //         }
    //         println!("val_read: {:?}", val_read);
    //         let request = String::from_utf8_lossy(&buffer[..val_read.unwrap()]);
    //         let body = request.splitn(2, "\r\n\r\n").nth(1).unwrap_or("");
    //         println!("request: {}", request);
    //         println!("body: {}", body);
    //         match handler(body) {
    //             Ok(msg) => {
    //                 let response = serde_json::to_string(&msg).unwrap();
    //                 let response = format!(
    //                     "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
    //                     response.len(),
    //                     response
    //                 );
    //                 stream.write(response.as_bytes()).unwrap();
    //             }
    //             Err(e) => {
    //                 let response = format!(
    //                     "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
    //                     e.len(),
    //                     e
    //                 );
    //                 stream.write(response.as_bytes()).unwrap();
    //             }
    //         }
    // }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(err) => {
                    println!("{}", err);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
