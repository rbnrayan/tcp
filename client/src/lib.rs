use std::io;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

// Uniques IDs

static COUNTER: AtomicUsize = AtomicUsize::new(0);
fn get_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }

// Client struct with:
// a unique ID
// a username
// the current connection with the server
pub struct Client {
    id: usize,
    username: String,
    conn: Arc<Mutex<TcpStream>>
}

impl Client {
    pub fn new(username: String, conn: Arc<Mutex<TcpStream>>) -> Self {
        Client {
            id: get_id(),
            username,
            conn,
        }
    }

    pub fn gen_id() -> usize {
        get_id()
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn ip(&self) -> IpAddr {
        self.conn.lock().unwrap().peer_addr().unwrap().ip()
    }

    pub fn port(&self) -> u16 {
        self.conn.lock().unwrap().peer_addr().unwrap().port()
    }

    // send data to the stream through the client
    pub fn send(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let nbytes = match self.conn.lock() {
            Ok(mut conn_lock) => {
                conn_lock.write(bytes)?
            }
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to lock the connection: {}", e),
                ))
            }
        };
        Ok(nbytes)
    }

    // receive data to the stream through the client
    pub fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nbytes = match self.conn.lock() {
            Ok(mut conn_lock) => {
                conn_lock.read(buf)?
            }
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to lock the connection: {}", e),
                ))
            }
        };
        Ok(nbytes)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        println!("[DISCONNECTED: Client {} ({})]", self.id, self.username);
        self.conn
            .lock()
            .unwrap()
            .shutdown(std::net::Shutdown::Both)
            .unwrap();
    }
}