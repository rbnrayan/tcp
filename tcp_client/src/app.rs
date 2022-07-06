use std::net::TcpStream;
use std::io::{self, Write};
use tcp_utils;

pub struct App<F>
where
    F: Fn(&[u8])
{
    stream: TcpStream,
    send_logfn: F,
}

impl<F> App<F>
where
    F: Fn(&[u8])
{
    pub fn new(stream: TcpStream, send_logfn: F) -> Self {
        App { stream, send_logfn }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let bytes_recvd = tcp_utils::read_bytes(&mut self.stream)?;
        println!("[Connected]\n\t{}\n", std::string::String::from_utf8(bytes_recvd).unwrap());

        let mut buf = String::new();
        loop {
            buf.clear();

            print!("[|>>|] : ");
            io::stdout().flush()?;
            io::stdin()
                .read_line(&mut buf)?;

            if let Some('\n') = buf.chars().next_back() {
                buf.pop();
            }

            tcp_utils::log_send_bytes(
                &mut self.stream,
                buf.as_bytes(),
                &self.send_logfn
            )?;

            println!();
        }
    }
}