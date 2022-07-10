use std::net::{TcpListener, TcpStream};
use std::io;
use std::sync::{Arc, RwLock};
use threadpool::{self, ThreadPool};
use tcp_utils::{self, Client};

const IP: &'static str = "127.0.0.1";
const PORT: &'static str = "8080";

fn handle_connection(stream: TcpStream, clients: Arc<RwLock<Vec<Arc<RwLock<Client>>>>>) -> io::Result<()> {
    let peer_connection = stream.peer_addr()?;
    // let read_logfn = |recvd_bytes: &Vec<u8>| println!(
    //     "[{}:{} |>>|] {{ {} bytes }} :\n\t`{}`",
    //     peer_connection.ip(),
    //     peer_connection.port(),
    //     recvd_bytes.len(),
    //     std::str::from_utf8(&recvd_bytes).unwrap(),
    // );
    // let send_logfn = |src: &[u8]| println!(
    //     "[{}:{} |<<|] {{ {} bytes }} :\n\t`{}`",
    //     peer_connection.ip(),
    //     peer_connection.port(),
    //     src.len(),
    //     std::str::from_utf8(src).unwrap(),
    // );

    let client_id = tcp_utils::get_id();
    let client_username = format!("user{}", clients.read().unwrap().len());
    let client = Arc::new(RwLock::new(Client::new(
        client_id,
        client_username,
        stream,
    )));
    clients.write().unwrap().push(Arc::clone(&client));

    println!(
        "[Connection established]\n\tIP_ADDR: {}\n\tPORT   : {}",
        peer_connection.ip(), 
        peer_connection.port()
    );

    client.write().unwrap().send(b"Connection with the server successful!")?;

    loop {
        let recvd_bytes = match client.write().unwrap().recv() {
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
        // let recvd_bytes = match tcp_utils::log_read_bytes(&mut stream, read_logfn) {
        //     Ok(bytes) => bytes,
        //     Err(_) => {
        //         println!(
        //             "[Connection closed]\n\tIP_ADDR: {}\n\tPORT   : {}",
        //             peer_connection.ip(),
        //             peer_connection.port()
        //         );
        //         break;
        //     }
        // };

        let username = format!("( {} )", client.read().unwrap().username());
        let msg = &[username.as_bytes(), &recvd_bytes].concat();

        println!("[Broadcast]\n\t`{}`", std::str::from_utf8(&recvd_bytes).unwrap());
        for client in clients.read().unwrap().iter() {
            client.write().unwrap().send(msg)?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", IP, PORT))?;
    let pool = ThreadPool::new(4, threadpool::Mode::Quiet);

    let clients = Arc::new(RwLock::new(Vec::new()));

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
