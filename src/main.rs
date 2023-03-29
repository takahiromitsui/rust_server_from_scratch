use rust_server::{MyTcpListener, ThreadPool, MyTcpStream};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct Message {
    room: String,
    username: String,
    message: String,
}

// fn handle_client(stream:MyTcpStream) {
//     let response = serve_html(stream, "src/views").unwrap();

// }

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = MyTcpListener::bind(addr.parse().unwrap())?;
    let pool = ThreadPool::new(5);
    let buffer = [0; 3000];
    let buffer_mutex = Arc::new(Mutex::new(buffer));
    loop {
        let mut stream = match listener.accept() {
            Ok(stream) => stream,
            Err(e) => {
                println!("Error accepting connection: {}", e);
                continue;
            }
        };
        let buffer_mutex_clone = Arc::clone(&buffer_mutex);
        pool.execute(move || {
            let buffer = &mut *buffer_mutex_clone.lock().unwrap();
            MyTcpListener::serve_html(buffer, &mut stream, "src/views").unwrap();
        })
    }
}
