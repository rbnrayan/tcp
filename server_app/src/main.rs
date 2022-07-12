use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use threadpool::{self, ThreadPool};
use client::Client;

const IP: &str = "127.0.0.1";
const PORT: &str = "8080";

fn handle_connection(stream: TcpStream, clients: Arc<RwLock<Vec<Client>>>) -> io::Result<()> {
    let stream_ref = Arc::new(Mutex::new(stream));
    let stream = Arc::clone(&stream_ref);
    let peer_addr = stream.lock().unwrap().peer_addr()?;

    // Server logs
    println!("[NEW CONNECTION]: {}:{}", peer_addr.ip(), peer_addr.port());

    if let Ok(mut stream) = stream.lock() {
        let greeting = b"Connected to the server!";
        let nbytes = stream.write(greeting)?;
        if nbytes != greeting.len() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Expected {} bytes to be written, found {}", greeting.len(), nbytes),
            ));
        }
    }

    let mut buf: [u8; 512] = [0; 512];
    // Server logs
    let read = if let Ok(mut stream) = stream.lock() {
        stream.read(&mut buf)
    } else {
        Ok(0)
    };
    match read {
        Ok(0) => println!("[CONNECTION CLOSED]: {}:{}", peer_addr.ip(), peer_addr.port()),
        Ok(nbytes) => {
            let username = if buf.starts_with(b"username: ") {
                let username: Vec<u8> = buf.into_iter()
                    .skip_while(|&c| c != b' ')
                    .collect();
                let mut username = username.into_iter();
                username.next();
                let username = username.collect();
                String::from_utf8(username).unwrap()
            } else {
                format!("user{}", Client::gen_id())
            };

            println!(
                "[RECV {}:{}]: '{}'",
                peer_addr.ip(),
                peer_addr.port(),
                std::str::from_utf8(&buf[..nbytes]).unwrap(),
            );

            let conn = Arc::clone(&stream_ref);
            let client = Client::new(username, conn);
            let client_id = client.id();

            println!(
                "[BIND USERNAME {} TO Client {}: {}:{}]",
                client.username(),
                client_id,
                peer_addr.ip(),
                peer_addr.port(),
            );

            if let Ok(mut clients_lock) = clients.write() {
                clients_lock.push(client);
            }

            loop {
                if let Ok(mut stream) = stream.lock() {
                    stream.set_nonblocking(true)?;
                    if let Ok(0) = stream.read(&mut buf) {
                        println!("[RECV] 0 bytes from Client {} ({}:{})", client_id, peer_addr.ip(), peer_addr.port());
                        break;
                    }
                }
                if let Ok(mut clients_lock) = clients.write() {
                    for client in clients_lock.iter_mut() {
                        if let Err(_) = client.send(b"server test...") {
                            println!("[FAILED TO SEND DATA TO {}:{}]", client.ip(), client.port());
                        }
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }

            if let Ok(mut clients_lock) = clients.write() {
                let i = clients_lock.iter().position(|c| c.id() == client_id);
                if let Some(i) = i {
                    clients_lock.remove(i);
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", IP, PORT))?;
    let pool = ThreadPool::new(5, threadpool::Mode::Quiet);

    let clients = Arc::new(RwLock::new(Vec::new()));

    println!("[SERVER INFO]\n\tServer listening on port: {}", PORT);

    for stream in listener.incoming() {
        let stream = stream?;
        let clients = Arc::clone(&clients);

        pool.execute(|| {
            if let Err(e) = handle_connection(stream, clients) {
                println!("[HANDLE CONN: ERROR]\n\t{}", e);
            }
        });
    }

    Ok(())
}
