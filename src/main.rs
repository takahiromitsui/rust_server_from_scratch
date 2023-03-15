use rust_server::MyTcpListener;
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    text: String,
    author: String,
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = MyTcpListener::bind(addr.parse().unwrap())?;

    loop {
        let mut stream = match listener.accept() {
            Ok(stream) => stream,
            Err(e) => {
                println!("Error accepting connection: {}", e);
                continue;
            }
        };
        let mut buffer = [0; 1024];
        let message = Message {
            text: "Hello, world!".to_string(),
            author: "Rust".to_string(),
        };
        MyTcpListener::serve_html(&mut buffer, &mut stream, "src/views")?;
        MyTcpListener::post_json(&mut buffer, &mut stream, "/message", &message)?;

        stream.flush()?;
    }
}
