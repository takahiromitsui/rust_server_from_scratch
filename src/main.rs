use std::net::TcpListener;

fn main() {
    let address = "127.0.0.1:5000";
    // 127.0.0.0.1 is the local host
    // bind is equivalent to new
    let listener = TcpListener::bind(address).unwrap();
    for stream in listener.incoming() {
        let _stream = stream.unwrap();
        println!("Connection established!");
    }

}
