use rust_server::{MyTcpListener, ThreadPool};
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
        let pool = ThreadPool::new(4);
        pool.execute(move || {
            let mut buffer = [0; 1024];
            MyTcpListener::serve_html(&mut buffer, &mut stream, "src/views").unwrap();
        })

        // MyTcpListener::post_json(&mut buffer, &mut stream, "/login", |json| {
        //     json["username"] = serde_json::Value::String("my_username".to_string());
        //     json["password"] = serde_json::Value::String("my_password".to_string());
        // }).unwrap();
        // stream.flush()?;
    }
}
