#[macro_use] extern crate log;
extern crate simplelog;
#[macro_use] extern crate lazy_static;

mod trips_and_tests;
mod tcp_stuff;

use trips_and_tests::*;
use tcp_stuff::*;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    //              R  u   s   t   meme
    //let address = "82.117.115.116::1337";
    //let listener = start_tcpstream(address);
    //let (sender, receiver) = mpsc::channel();
    tg_log_init();
    let (send_inst, recv_inst): (mpsc::Sender<TGInst>, mpsc::Receiver<TGInst>) = mpsc::channel();
    let (data_send, data_recv): (mpsc::Sender<TGTReturn>, mpsc::Receiver<TGTReturn>) = mpsc::channel();
    let m_generator = TripGenMain::new(send_inst, data_recv);
    let t_generator = TripGenThread::new(recv_inst, data_send, 100_000);
    let test = m_generator.get_data(10);
    let test2 = m_generator.pause();
    match test2 {
        Ok(_) => println!("Paused successfully."),
        Err(tgerr) => {
            println!("TGError.");
            log_tgerror(tgerr);
        },
        Err(other) => println!("Other error: {:?}", other)
    }
    println!("Result: {:?}", test);
    println!("Done.");
}

//   _____              _ _ _
//  / ____|            | (_) |
// | |     _ __ ___  __| |_| |_ ___
// | |    | '__/ _ \/ _` | | __/ __|
// | |____| | |  __/ (_| | | |_\__ \
//  \_____|_|  \___|\__,_|_|\__|___/
// No thanks to:
// - MelodicStream (Special™)
// - Oberien
// - Repnop
// - Flying Janitor
// Useless gits >:v