#[macro_use] extern crate log;
extern crate simplelog;

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
    let generator = TripGen::new(10_000_000);
    let a_generator = Arc::new(Mutex::new(generator));
    /*let mut threads = vec![];
    for no in 0..3 {
        let t_generator = Arc::clone(&a_generator);
        let request_len = 512;
        let checker = thread::spawn(move || {
            loop {
                let data = ParkerByParts::mt_get_trips(&t_generator, request_len);
                match data {
                    Ok(trips) => {
                        for trip in trips.into_iter() {
                            if ParkerByParts::test_squares(trip) {
                                println!("Hory shet! Solution: {:?}", trip);
                            }
                        }
                    },
                    Err(TGError::NotActive) => {
                        log_tgerror(no, TGError::NotActive);
                        break;
                    },
                    Err(tgerr) => {
                        log_tgerror(no, tgerr);
                        continue;
                    }
                }
            }
            println!("Thread {} finished execution.", no);
        });
        threads.push(checker);
    }
    for a in threads {
        a.join().unwrap();
    }*/
    let start = std::time::SystemTime::now();
    let mut count = 0;
    let t_generator = Arc::clone(&a_generator);
    let request_len = 10_000;
    for _ in 0..250 {
        let data = mt_get_trips(&t_generator, request_len);
        match data {
            Ok(trips) => {
                for trip in trips.into_iter() {
                    if test_squares(trip) {
                        println!("Hory shet! Solution: {:?}", trip);
                    }
                }
            },
            Err(TGError::NotActive) => {
                log_tgerror(0, TGError::NotActive);
                break;
            },
            Err(tgerr) => {
                log_tgerror(0, tgerr);
                continue;
            }
        }
        count += 1;
    }
    println!("{:?}", start.elapsed().unwrap());
    println!("Received data {} times.", count);
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