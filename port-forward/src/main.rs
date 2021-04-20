// REF: https://github.com/karlvlam/rustforward
use std::env;
use std::error::Error;
use std::io::{self, Read, Write};
use std::net::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

fn main() {
    let mut args = env::args().skip(1);
    let (src, dest) = match (args.next(), args.next()) {
        (Some(src), Some(dest)) => (src, dest),
        _ => {
            show_help();
            return;
        }
    };
    start_listener(&src, &dest).unwrap();
}

fn show_help() {
    println!("Usage: rustforward portforwardlist.conf");
}

struct TcpBuffer {
    data: [u8; 128],
    length: usize,
}

fn start_listener(src_addr: &str, dest_addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(src_addr)?;
    println!("Port forward started {} -> {}", src_addr, dest_addr);
    for stream in listener.incoming() {
        let addr = dest_addr.clone().to_owned();
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream, &addr);
                });
            }
            Err(_) => {
                println!("sth error!");
            }
        }
    }
    Ok(())
}

fn write_stream(stream: &mut TcpStream, rx: &Receiver<TcpBuffer>) -> io::Result<()> {
    while let Ok(TcpBuffer { data, length }) = rx.try_recv() {
        match stream.write(&data[0..length]) {
            Ok(v) => {
                println!("Writed {} bytes", v);
            }
            Err(e) => {
                println!("Error writing: {}", e);
            }
        };
    }
    Ok(())
}

fn read_stream(stream: &mut TcpStream, tx: &Sender<TcpBuffer>) -> Result<(), Box<dyn Error>> {
    let mut buf: [u8; 128] = [0; 128];
    loop {
        match stream.read(&mut buf) {
            Ok(byte_count) => {
                println!("Readed {} bytes.", byte_count);
                tx.send(TcpBuffer {
                    data: buf,
                    length: byte_count,
                })?;
                if byte_count < buf.len() {
                    break;
                }
            }
            Err(e) => {
                println!("Error reading: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}

fn handle_client(mut src_stream: TcpStream, dest_addr: &str) {
    let (dest_tx, dest_rx): (Sender<TcpBuffer>, Receiver<TcpBuffer>) = channel();
    let (src_tx, src_rx): (Sender<TcpBuffer>, Receiver<TcpBuffer>) = channel();

    let mut dest_stream = TcpStream::connect(dest_addr).unwrap();
    // src_stream.set_nonblocking(true).unwrap();
    // dest_stream.set_nonblocking(true).unwrap();

    println!("Source reading.");
    read_stream(&mut src_stream, &dest_tx).unwrap();
    println!("Dest writing.");
    write_stream(&mut dest_stream, &dest_rx).unwrap();
    println!("Dest reading.");
    read_stream(&mut dest_stream, &src_tx).unwrap();
    println!("Source writing.");
    write_stream(&mut src_stream, &src_rx).unwrap();

    src_stream.shutdown(Shutdown::Both).unwrap();
    dest_stream.shutdown(Shutdown::Both).unwrap();
}
