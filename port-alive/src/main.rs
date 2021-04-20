use std::env;
use std::net::{TcpStream, ToSocketAddrs};

fn main() {
    let addrs = env::args()
        .nth(1)
        .unwrap()
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    if TcpStream::connect(addrs).is_ok() {
        println!("Port is open.");
    } else {
        println!("Port is closed");
    };
}
