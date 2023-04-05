use std::net::TcpListener;

use rust_server::{MyTcpListener, ThreadPool};

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr)?;
    // let listener = MyTcpListener::bind(addr.parse().unwrap())?;
    let pool = ThreadPool::new(4);

    loop {
        let stream = match listener.accept() {
            Ok(stream) => stream,
            Err(e) => {
                println!("Error accepting connection: {}", e);
                continue;
            }
        };
        pool.execute(move || {
            MyTcpListener::handle_connection(stream.0, "src/views");
        })
    }
    // loop {
    //     let mut stream = match listener.accept() {
    //         Ok(stream) => stream,
    //         Err(e) => {
    //             println!("Error accepting connection: {}", e);
    //             continue;
    //         }
    //     };
    //     pool.execute(move || {
    //         MyTcpListener::serve_html(&mut stream, "src/views").unwrap();
    //     })
    // }
}
