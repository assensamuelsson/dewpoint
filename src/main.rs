use std::net::TcpListener;
use std::env;

fn main() {
    let port = env::var("PORT").unwrap_or("5000".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                dewpoint::handle_connection(stream)
            },
            Err(_) => eprintln!("Could not get a stream from the incoming connection!"),
        }
    }
}
