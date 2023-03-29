use rust_server::{MyTcpListener, ThreadPool};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct Message {
    text: String,
    author: String,
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = MyTcpListener::bind(addr.parse().unwrap())?;
    let pool = ThreadPool::new(4);
    loop {
        let mut stream = match listener.accept() {
            Ok(stream) => stream,
            Err(e) => {
                println!("Error accepting connection: {}", e);
                continue;
            }
        };
        pool.execute(move || {
            MyTcpListener::serve_html(&mut stream, "src/views").unwrap();
        })
    }
}
