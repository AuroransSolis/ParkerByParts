extern crate ParkerByParts;

use ParkerByParts::TripGen;
use ParkerByParts::TGError;
use ParkerByParts::tg_log_init;
use ParkerByParts::log_tgerror;

use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    tg_log_init();
    let generator = TripGen::new(65_536);
    let a_generator = Arc::new(Mutex::new(generator));
    let mut threads = vec![];
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
    }
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