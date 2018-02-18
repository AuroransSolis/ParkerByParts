extern crate asdfasdf;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use asdfasdf::TripFetcher;

fn main() {
    let fetcher = TripFetcher::new(16_384, (0, 0, 0));
    let fetcher = Arc::new(Mutex::new(fetcher));
    let mut threads = vec![];
    for no in 0..3 {
        let fetcher = Arc::clone(&fetcher);
        let request_len = (no + 1) * (no + 1) * 100;
        let checker = thread::spawn(move || {
            loop {
                let mut m_fetcher = fetcher.try_lock();
                match m_fetcher {
                    Ok(ref mut t_fetcher) => {
                        if !t_fetcher.active {
                            break;
                        }
                        let trips = t_fetcher.get_triplets_vec(request_len);
                        if let Err(tferr) = trips {
                            println!("{}", tferr);
                            continue;
                        }
                        let trips = trips.unwrap();
                        println!("Thread {} got {} trips | Start: {:?}, end: {:?}", no, trips.len(), trips[0], trips[trips.len() - 1]);
                        for trip in trips.into_iter() {
                            if asdfasdf::test_squares(trip) {
                                println!("Hory shet! Solution: {:?}", trip);
                            }
                        }
                    },
                    _ => continue
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