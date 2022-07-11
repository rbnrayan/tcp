use std::net::{TcpStream, IpAddr};
use std::io::{self, Write};
use tcp_utils;

pub struct App<F>
where
    F: Fn(&[u8], IpAddr, u16)
{
    stream: TcpStream,
    logs: bool,
    send_logfn: Option<F>,
}

impl<F> App<F>
where
    F: Fn(&[u8], IpAddr, u16)
{
    pub fn new(stream: TcpStream, send_logfn: Option<F>) -> Self {
        App { 
            stream,
            logs: false,
            send_logfn
        }
    }

    #[allow(dead_code)]
    pub fn logs(&mut self, b: bool) {
        self.logs = b;
    }

    pub fn run(&mut self) -> io::Result<()> {
        //let bytes_recvd = tcp_utils::read_bytes(&mut self.stream)?;
        //println!("[Connected]\n\t{}\n", std::string::String::from_utf8(bytes_recvd).unwrap());

        self.stream.set_nonblocking(true)?;

        let mut buf = String::new();
        loop {
            match tcp_utils::read_bytes(&mut self.stream, |_, _, _,| {}) {
                Ok(recvd_bytes) => println!(" -> {}", std::string::String::from_utf8(recvd_bytes).unwrap()),
                Err(_) => {}
            }

            buf.clear();

            print!("[|>>|] : ");
            io::stdout().flush()?;
            io::stdin()
                .read_line(&mut buf)?;

            if let Some('\n') = buf.chars().next_back() {
                buf.pop();
            }
            if buf.is_empty() {
                continue;
            }

            if self.logs {
                tcp_utils::send_bytes(
                    &mut self.stream,
                    buf.as_bytes(),
                    self.send_logfn.as_ref().unwrap()
                )?;
                println!();
            } else {
                tcp_utils::send_bytes(&mut self.stream, buf.as_bytes(), |_, _, _| {})?;
            }
        }
    }
}
