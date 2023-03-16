use rust_server::MyTcpListener;

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
        MyTcpListener::serve_html(&mut buffer,&mut stream, "src/views")?;

        stream.flush()?;
    }
}
