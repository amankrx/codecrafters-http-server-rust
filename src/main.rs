mod connection_handler;
mod error;
mod models;

use crate::connection_handler::handle_connection;
use std::net::TcpListener;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // WARNING: This can create unlimited number of thread pools.
                thread::spawn(|| {
                    handle_connection(stream).expect("TODO: panic message");
                });
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}
