use std::net::{TcpListener, TcpStream};
use std::io;
use std::sync::{Arc, Mutex};
use threadpool::{self, ThreadPool};
use tcp_utils;

const IP: &'static str = "127.0.0.1";
const PORT: &'static str = "8080";

fn handle_connection(mut stream: TcpStream, clients: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>>) -> io::Result<()> {
    let peer_connection = stream.peer_addr()?;
    let read_logfn = |recvd_bytes: &Vec<u8>| println!(
        "[{}:{} |>>|] {{ {} bytes }} :\n\t`{}`",
        peer_connection.ip(),
        peer_connection.port(),
        recvd_bytes.len(),
        std::str::from_utf8(&recvd_bytes).unwrap(),
    );
    let send_logfn = |src: &[u8]| println!(
        "[{}:{} |<<|] {{ {} bytes }} :\n\t`{}`",
        peer_connection.ip(),
        peer_connection.port(),
        src.len(),
        std::str::from_utf8(src).unwrap(),
    );

    let client = Arc::new(Mutex::new(stream.try_clone()?));
    clients.lock().unwrap().push(Arc::clone(&client));

    println!(
        "[Connection established]\n\tIP_ADDR: {}\n\tPORT   : {}",
        peer_connection.ip(), 
        peer_connection.port()
    );

    let msg = b"Connection with the server successful!";
    tcp_utils::log_send_bytes(&mut stream, msg, send_logfn)?;

    loop {
        let read_bytes = match tcp_utils::log_read_bytes(&mut stream, read_logfn) {
            Ok(bytes) => bytes,
            Err(_) => {
                println!(
                    "[Connection closed]\n\tIP_ADDR: {}\n\tPORT   : {}",
                    peer_connection.ip(),
                    peer_connection.port()
                );
                break;
            }
        };
        
        let clients_lock = clients.lock().unwrap();
        let clients = clients_lock.iter();
        for client in clients {
            let mut client = client.lock().unwrap();
            tcp_utils::log_send_bytes(&mut client, &read_bytes, send_logfn)?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", IP, PORT))?;
    let pool = ThreadPool::new(4, threadpool::Mode::Quiet);

    let clients = Arc::new(Mutex::new(Vec::new()));

    println!("[Server info]\n\tIP_ADDR: {}\n\tPORT   : {}\n", IP, PORT);

    for stream in listener.incoming() {
        let stream = stream?;
        let clients = Arc::clone(&clients);

        pool.execute(|| {
            if let Err(e) = handle_connection(stream, clients) {
                println!("[HANDLE ERROR]:\n\t{}", e);
            }
        });
    }

    Ok(())
}
