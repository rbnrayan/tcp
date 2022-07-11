use std::net::{TcpStream, IpAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::io::{self, Read, Write};

pub fn send_bytes<F>(stream: &mut TcpStream, src: &[u8], logfn: F) -> io::Result<()> 
where
    F: Fn(&[u8], IpAddr, u16),
{
    let mut data = Vec::from(src);
    data.push(b'\0');

    let bytes_written = stream.write(&data)?;

    if data.len() != bytes_written {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "`tcp_utils::send_bytes`: expected {} bytes to be written, found {}",
                data.len(),
                bytes_written
            )
        ))
    }
    let peer_addr = stream.peer_addr()?;
    logfn(src, peer_addr.ip(), peer_addr.port());

    Ok(())
}

pub fn read_bytes<F>(stream: &mut TcpStream, logfn: F) -> io::Result<Vec<u8>>
where
    F: Fn(&Vec<u8>, IpAddr, u16),
{
    let mut buf = [0; 128];
    let mut bytes = Vec::new();
    loop {
        match stream.read(&mut buf) {
            Ok(_) => {
                if buf.iter().all(|&b| b != b'\0') {
                    bytes.extend_from_slice(&buf);
                } else {
                    let truncated_buf = buf.into_iter()
                        .take_while(|&b| b != b'\0')
                        .collect::<Vec<u8>>();
                    bytes.extend_from_slice(&truncated_buf[..]);
                    break;
                }
            }
            Err(e) => return Err(
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("`tcp_utils::read_bytes`: {}", e),
                )
            ),
        }
    }
    let peer_addr = stream.peer_addr()?;
    logfn(&bytes, peer_addr.ip(), peer_addr.port());
    Ok(bytes)
}

// Uniques IDs

static COUNTER: AtomicUsize = AtomicUsize::new(0);
pub fn get_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }

// Client struct
#[derive(Debug)]
pub struct Client {
    id: usize,
    username: String,
    conn: TcpStream,
}

impl Client {
    pub fn new(id: usize, username: String, conn: TcpStream) -> Self {
        Client {
            id,
            username,
            conn,
        }
    }

    pub fn recv<F>(&mut self, f: F) -> io::Result<Vec<u8>>
    where
        F: Fn(&Vec<u8>, IpAddr, u16),
    {
        read_bytes(&mut self.conn, f)
    }

    pub fn send<F>(&mut self, bytes: &[u8], f: F) -> io::Result<()>
    where
        F: Fn(&[u8], IpAddr, u16),
    {
        send_bytes(&mut self.conn, bytes, f)
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn conn(&self) -> &TcpStream {
        &self.conn
    }
}
