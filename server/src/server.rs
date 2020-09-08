use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use super::message;
use message::{RecvMessage, SendMessage};

#[derive(Debug)]
enum ServerErrors {
    MissedConnection,
}
impl fmt::Display for ServerErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
impl Error for ServerErrors {}

pub struct Server {
    socket: SocketAddrV4,
    listener: TcpListener,
    tcp_stream: Option<TcpStream>,
}

impl Server {
    pub fn new(addr: Ipv4Addr, port: u16) -> Result<Server, Box<dyn Error>> {
        let socket = SocketAddrV4::new(addr, port);
        Ok(Server {
            socket: socket,
            listener: TcpListener::bind(socket)?,
            tcp_stream: None,
        })
    }
    pub fn wait_client(&mut self) -> Result<(), Box<dyn Error>> {
        let port = self.listener.local_addr()?;
        println!("Listening on {}, access this port from a client", port);
        let (tcp_stream, client_addr) = self.listener.accept()?; //block  until requested
        self.tcp_stream = Some(tcp_stream);
        println!("Connection received! {:?}", client_addr);
        Ok(())
    }

    pub fn send(&mut self, msg: Box<dyn SendMessage>) -> Result<(), Box<dyn Error>> {
        let mut msg = msg;
        match &mut self.tcp_stream {
            Some(stream) => {
                let mut id_size: [u8; 5] = [0; 5];
                id_size[0] = msg.id();
                let size_bytes = msg.size().to_be_bytes();
                id_size[1..].copy_from_slice(&size_bytes);
                stream.write_all(&id_size)?;
                let bytes = msg.to_bytes();
                match bytes {
                    Some(bytes) => {
                        stream.write_all(bytes)?;
                        return Ok(());
                    }
                    None => return Ok(()),
                }
            }
            None => Err(Box::new(ServerErrors::MissedConnection)),
        }
    }

    pub fn recv<T: RecvMessage>(&mut self, msg: &mut T) -> Result<(), Box<dyn Error>> {
        match &mut self.tcp_stream {
            Some(stream) => {
                let mut id_size: [u8; 5] = [0; 5];
                stream.read_exact(&mut id_size)?;
                let tmp = slice_as_array!(&id_size[1..], [u8; 4])
                    .expect("Server::recv wrong header data");
                let size = u32::from_be_bytes(*tmp);
                Server::read_msg::<T>(msg, stream, size as usize)?;
                Ok(())
            }
            None => Err(Box::new(ServerErrors::MissedConnection)),
        }
    }

    fn read_msg<T: RecvMessage>(
        msg: &mut T,
        stream: &mut TcpStream,
        size: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(size, 0);
        stream.read_exact(&mut buf)?;
        msg.from_bytes(&buf);
        Ok(())
    }
}
