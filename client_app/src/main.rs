use std::net::TcpStream;
use std::io;
use std::io::{Read, Write};
use client::Client;

fn main() -> io::Result<()> {
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(mut stream) => {
            let mut buf: [u8; 512] = [0; 512];
            let nbytes = stream.read(&mut buf)?;

            println!(":-> {}", std::str::from_utf8(&buf[..nbytes]).unwrap());

            let username = format!("username: custom_username{}", Client::gen_id());
            let write_nbytes = stream.write(username.as_bytes())?;
            if write_nbytes != username.len() {
                return Err(io::Error::new(io::ErrorKind::Other, format!("Expected {} bytes to be written, found {}", write_nbytes, username.len())));
            }

            loop {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => { break; }
                    Ok(nbytes) => {
                        println!(":-> {}", std::str::from_utf8(&buf[..nbytes]).unwrap());
                    }
                }
            }
        },
        Err(_) => println!("Failed to connect to the server"),
    }
    Ok(())
}
