use std::net::TcpStream;
use std::io::{self, Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use client::Client;

fn main() -> io::Result<()> {
    // match on the connection to the server
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(stream) => {
            let stream = Arc::new(Mutex::new(stream));

            let mut buf: [u8; 512] = [0; 512];

            match stream.lock() {
                Ok(mut stream_lock) => {
                    // set a buffer to read incoming data from the server
                    let nbytes = stream_lock.read(&mut buf)?;

                    // show the received data
                    println!(":-> {}", std::str::from_utf8(&buf[..nbytes]).unwrap());

                    // set the username and send it to the server
                    // then check the number of bytes sended to the server
                    // assert!(sended_bytes == username.len());
                    let username = format!("username: _display_{}", Client::gen_id());
                    let nbytes = stream_lock.write(username.as_bytes())?;
                    if nbytes != username.len() {
                        return Err(io::Error::new(io::ErrorKind::Other, format!("Expected {} bytes to be written, found {}", nbytes, username.len())));
                    }
                },
                Err(e) => { println!("Failed to get a lock of the stream: {}", e); std::process::exit(1); },
            }


            let read_thread_stream = Arc::clone(&stream);
            read_thread_stream.lock().unwrap().set_nonblocking(true)?;

            // loop through incoming data from the server
            thread::spawn(move || {
                loop {
                    match read_thread_stream.lock().unwrap().read(&mut buf) {
                        Ok(0) => { break; },
                        Ok(nbytes) => println!("\nbuf -> {}", std::str::from_utf8(&buf[..nbytes]).unwrap()),
                        Err(_) => {},
                    }
                    thread::sleep(std::time::Duration::from_millis(500));
                }
            });

            let mut input = String::new();

            loop {
                input.clear();
                print!("input: ");
                io::stdout().flush()?;
                io::stdin().read_line(&mut input).unwrap();
                if input.starts_with(":q") { std::process::exit(0); }
                let _ = stream.lock().unwrap().write(input.as_bytes());
                thread::sleep(std::time::Duration::from_millis(1000));
            }
        },
        Err(_) => println!("Failed to connect to the server"),
    }
    Ok(())
}
