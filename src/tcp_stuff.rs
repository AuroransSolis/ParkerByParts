use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

pub fn start_tcpstream(addr: String) -> TcpListener {
    TcpListener::bind(addr).unwrap()
}

