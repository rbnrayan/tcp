use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use threadpool::{self, ThreadPool};
use client::Client;

// constant addr to listen to
const IP: &str = "127.0.0.1";
const PORT: &str = "8080";

fn handle_connection(stream: TcpStream, clients: Arc<RwLock<Vec<Client>>>) -> io::Result<()> {
    // transform stream to a Arc<Mutex<Stream>> to be shared
    // accross this function and the client::Client struct
    let stream_ref = Arc::new(Mutex::new(stream));
    let stream = Arc::clone(&stream_ref);

    // storing the peer_addr of the stream
    let peer_addr = stream.lock().unwrap().peer_addr()?;

    // server log
    println!("[NEW CONNECTION]: {}:{}", peer_addr.ip(), peer_addr.port());

    // if we got a lock of the stream send a greeting message to the client
    if let Ok(mut stream) = stream.lock() {
        let greeting = b"Connected to the server!";
        let nbytes = stream.write(greeting)?;

        // check if we send the right number of bytes
        if nbytes != greeting.len() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Expected {} bytes to be written, found {}", greeting.len(), nbytes),
            ));
        }
    }

    // creating a buffer to store incoming data from the stream
    let mut buf: [u8; 512] = [0; 512];

    // if we got a lock of the stream, read the incoming data
    // and get back the number of bytes readed
    let read = match stream.lock() {
        Ok(mut stream) => {
            stream.read(&mut buf)
        },
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to get a lock from the stream: {}", e),
            ));
        }
    };

    // match the result of the stream.read() above
    match read {
        // server logs
        Ok(0) => println!("[CONNECTION CLOSED]: {}:{}", peer_addr.ip(), peer_addr.port()),
        Ok(nbytes) => {
            // read the incoming data (a message for setting the username)
            // -> 'username: <username>'
            // if it match the format then get the username passed
            // else get a basic username (user0, user1, ...)
            let username = if buf.starts_with(b"username: ") {
                let mut username = buf.into_iter()
                    .skip_while(|&c| c != b' ');
                username.next();
                let username = username.collect();
                String::from_utf8(username).unwrap()
            } else {
                format!("user{}", Client::gen_id())
            };

            // server log
            println!(
                "[RECV {}:{}]: '{}'",
                peer_addr.ip(),
                peer_addr.port(),
                std::str::from_utf8(&buf[..nbytes]).unwrap(),
            );

            // clone the stream and pass it along the username to create a new client::Client
            let conn = Arc::clone(&stream_ref);
            let client = Client::new(username, conn);

            // store the client ID for retriving at the end
            let client_id = client.id();

            // server log
            println!(
                "[BIND USERNAME {} TO Client {}: {}:{}]",
                client.username(),
                client_id,
                peer_addr.ip(),
                peer_addr.port(),
            );

            // if we get a lock of the clients (list of the connected clients)
            // push the current client into the list
            if let Ok(mut clients_lock) = clients.write() {
                clients_lock.push(client);
            }

            loop {
                // get a lock of the stream, set it to be non blocking
                // and see if when we read it the size returned is 0
                // if it is, the client is probably disconnected
                if let Ok(mut stream) = stream.lock() {
                    stream.set_nonblocking(true)?;
                    if let Ok(0) = stream.read(&mut buf) {
                        // println!("[RECV] 0 bytes from Client {} ({}:{})", client_id, peer_addr.ip(), peer_addr.port());
                        break;
                    }
                }
                // get a lock of the clients list and iter over
                // send data for each of the clients in the list
                if let Ok(mut clients_lock) = clients.write() {
                    for client in clients_lock.iter_mut() {
                        if client.username().starts_with("_display_") {
                            let _ = client.send(b"server test...");
                        }
                    }
                }
            }

            // get a mut lock of the clients list then retrieve
            // the client of the current thread (with client ID)
            // then remove it from the clients list
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
    // set up the socket and the thread pool
    let listener = TcpListener::bind(format!("{}:{}", IP, PORT))?;
    let pool = ThreadPool::new(5, threadpool::Mode::Quiet);

    // a vector of client::Client that can be shared between threads with Arc<...>
    let clients = Arc::new(RwLock::new(Vec::new()));

    // server log
    println!("[SERVER INFO]\n\tServer listening on port: {}", PORT);

    // for each incoming connection call `handle_connection()`
    // with the stream of the connection and a clone of the clients
    // in a thread from the pool
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
