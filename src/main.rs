#[macro_use] extern crate log;
extern crate simplelog;

mod trips_and_tests;
mod tcp_stuff;

use trips_and_tests::*;
use tcp_stuff::*;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    //                R  u   s   t   meme
    //let address = "82.117.115.116::1337";
    //let listener = start_tcpstream(address);
    //let (sender, receiver) = mpsc::channel();
    tg_log_init();
    let (send_inst, recv_inst): (mpsc::Sender<TGInst>, mpsc::Receiver<TGInst>) = mpsc::channel();
    let (data_send, data_recv): (mpsc::Sender<TGTReturn>, mpsc::Receiver<TGTReturn>) = mpsc::channel();
    let mut m_generator = TripGenMain::new(send_inst, data_recv);
    let t_generator = TripGenThread::new(recv_inst, data_send, 10_000_000, 50_000_000);
    let tgen_handle = run(t_generator);
    let wait_time = 300;
    println!("Allowing TGen to fill buffer. Waiting for: {} seconds.", wait_time);
    //thread::sleep(std::time::Duration::from_secs(wait_time));
    for _ in 0..6 {
        thread::sleep(std::time::Duration::from_secs(30));
        match m_generator.query_buffer_size() {
            Ok(size) => println!("Buffer size at the end of the wait period: {}", size),
            Err(e) => println!("Error getting buffer size: {:?}", e)
        }
    }
    match m_generator.query_buffer_size() {
        Ok(size) => println!("Buffer size at the end of the wait period: {}", size),
        Err(e) => println!("Error getting buffer size: {:?}", e)
    }
    match m_generator.progress() {
        Ok(trip) => println!("Last trip in buf at start: {:?}", trip),
        Err(e) => println!("Error getting progress: {:?}", e)
    }
    println!("Starting test.");
    let start = std::time::SystemTime::now();
    let mut times = 0;
    for _ in 0..1000 {
        let asdf = m_generator.get_data(10_000);
        times += 1;
    }
    println!("{:?}", start.elapsed().unwrap());
    println!("Received data {} times.", times);
    tgen_handle.join().unwrap();
}

//   _____              _ _ _        //
//  / ____|            | (_) |       //
// | |     _ __ ___  __| |_| |_ ___  //
// | |    | '__/ _ \/ _` | | __/ __| //
// | |____| | |  __/ (_| | | |_\__ \ //
//  \_____|_|  \___|\__,_|_|\__|___/ //
// No thanks to:                     //
// - MelodicStream (Special™)       //
// - Oberien                         //
// - Repnop                          //
// - Flying Janitor                  //
// Useless gits >:v                  //