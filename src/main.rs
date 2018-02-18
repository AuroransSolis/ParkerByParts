extern crate asdfasdf;
use std::thread;
use std::sync::{Arc, Mutex};

use asdfasdf::TripFetcher;
use asdfasdf::TFError;

fn main() {
    let fetcher = TripFetcher::new(100_000, (0, 0, 0));
    let fetcher = Arc::new(Mutex::new(fetcher));
    let mut threads = vec![];
    for no in 0..3 {
        let fetcher = Arc::clone(&fetcher);
        let request_len = 1000;
        let checker = thread::spawn(move || {
            loop {
                let data = asdfasdf::mt_get_trips(&fetcher, request_len);
                match data {
                    Ok(trips) => {
                        for trip in trips.into_iter() {
                            if asdfasdf::test_squares(trip) {
                                println!("Hory shet! Solution: {:?}", trip);
                            }
                        }
                    },
                    Err(tferr) => {
                        match tferr {
                            TFError::NotActive => break,
                            _ => {
                                println!("Error: {}", tferr);
                                continue;
                            }
                        }
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