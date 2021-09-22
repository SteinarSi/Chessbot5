use std::thread::*;
use std::net::*;
use std::io::*;
use crate::backend::board_representation::movement::*;
use serde::{Serialize, Deserialize};

pub struct Connection{
    stream:TcpStream

}

impl Connection{
    pub fn new(ip:String, host:bool) -> Self
    {
        if host
        {
            let listener = TcpListener::bind(ip).unwrap();
            let (conn, _) = listener.accept().unwrap();
            Connection{stream:conn}//TODO: Known bug, will block main until it connects
        }
        else
        {
            Connection{stream:TcpStream::connect(ip).unwrap()}
        }
    }

    pub fn send_move(&mut self, mve:&str)->bool
    {
        match self.stream.write_all(mve.as_bytes()){
            Ok(()) => true,
            Err(_error) => false,
        }
    }
    pub async fn recv_move(&mut self)->Vec<u8>
    {
        let mut buf = vec![];
        self.stream.read_to_end(&mut buf);
        let retBuf = buf;
        return retBuf;
    }
    
}