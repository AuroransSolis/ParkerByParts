#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;
use std::fs::File;

use std::sync::{Arc, Mutex};

pub struct TripGen {
    pub max: u64,
    pub start: (usize, usize, usize),
    pub active: bool
}

#[derive(Debug)]
pub enum TGError {
    InvalidStart,
    NotActive,
    EmptyReturn,
}

pub fn tg_log_init() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("magic_search.log").unwrap())
        ]
    ).unwrap();
}

pub fn log_tgerror(thread: usize, error: TGError) {
    match error {
        TGError::InvalidStart => info!("Thread {} passed invalid start tuple.", thread),
        TGError::NotActive => error!("Thread {} attempted to use inactive triplet generator.", thread),
        TGError::EmptyReturn => info!("Thread {} got empty return vec.", thread)
    }
}

impl TripGen {
    pub fn new(max: u64) -> Self {
        TripGen {
            max: max,
            start: (0, 0, 0),
            active: true
        }
    }

    // 
    pub fn get_triplets_vec(&mut self, vec_len: usize) -> Result<Vec<(u64, u64, u64)>, TGError> {
        let max = self.max;
        let start_trips = self.start;
        // Return None if any of the start conditions are too high
        if start_trips.0 > max as usize || start_trips.1 > max as usize || start_trips.2 > max as usize {
            return Err(TGError::InvalidStart);
        }
        if !self.active {
            return Err(TGError::NotActive);
        }
        // Make vec to be returned
        let mut new_trips = Vec::new();
        let mut y_skip = start_trips.1;
        let mut z_skip = start_trips.2;
        let mut x_skip = 0;
        let mut iter = true;
        for x_num in (0u64..).skip(start_trips.0).take_while(|x| x * x < max) {
            if !iter {
                break;
            }
            let x = x_num * x_num;
            x_skip = x_num;
            for y in (0..x).skip(y_skip) {
                if !iter {
                    break;
                }
                for z in (0..x - y).skip(z_skip) {
                    if x > 0 && y > 0 && z > 0  && y != z && all_valid((x, y, z)) && new_trips.len() < vec_len {
                        new_trips.push((x, y, z));
                    }
                    if new_trips.len() == vec_len {
                        iter = false;
                        break;
                    }
                }
                // Only skip on first iteration
                z_skip = 0;
            }
            // Only skip on first iteration
            y_skip = 0;
        }
        // Even with valid inputs the return vec len can be 0 - return None if it is
        if new_trips.len() == 0 {
            return Err(TGError::EmptyReturn);
        }
        // Deactivate generator if the return vec length is not the requested length. The only time this will happen is if
        // there aren't enough to meet the requested length at the end of the iterators.
        if new_trips.len() != vec_len {
            self.active = false;
        }
        // Set new last element
        let last_elem = new_trips[new_trips.len() - 1];
        self.start = (x_skip as usize, last_elem.1 as usize, (last_elem.2 + 1) as usize);
        //println!("New start: {:?}", self.start);
        Ok(new_trips)
    }
}

pub fn mt_get_trips(am_tg: &Arc<Mutex<TripGen>>, request_len: usize) -> Result<Vec<(u64, u64, u64)>, TGError> {
    let mut m_generator = am_tg.try_lock();
    let mut data = Vec::new();
    if let Ok(ref mut t_generator) = m_generator {
        if !t_generator.active {
            return Err(TGError::NotActive);
        }
        let trips = t_generator.get_triplets_vec(request_len);
        if let Err(tgerr) = trips {
            return Err(tgerr);
        }
        data = trips.unwrap();
    }
    drop(m_generator);
    Ok(data)
}

// Only use triplet (x, y, z) if the squared values of combinations of them are all 1 mod 24
fn all_valid(trip: (u64, u64, u64)) -> bool {
    (trip.0 + trip.1) % 24 == 1 && (trip.0 - trip.1 - trip.2) % 24 == 1 && (trip.0 + trip.2) % 24 == 1
        && (trip.0 - trip.1 + trip.2) % 24 == 1 && trip.0 % 24 == 1 && (trip.0 + trip.1 - trip.2) % 24 == 1
        && (trip.0 - trip.2) % 24 == 1 && (trip.0 + trip.1 + trip.2) % 24 == 1 && (trip.0 - trip.1) % 24 == 1
}

// Using method found on SE
const GOOD_MASK: u64 = 0xC840C04048404040;

fn is_valid_square(mut n: u64) -> bool {
    if n % 24 != 1 {
        return false;
    }
    if (GOOD_MASK << n) as i64 >= 0 {
        return false;
    }
    let zeros = n.trailing_zeros();
    if zeros & 1 != 0 {
        return false;
    }
    n >>= zeros;
    if n & 7 != 1 {
        return n == 0;
    }
    ((n as f64).sqrt() as u64).pow(2) == n
}

// Test the combinations of them to see if they're square.
pub fn test_squares(trip: (u64, u64, u64)) -> bool {
    is_valid_square(trip.0 + trip.1) && is_valid_square(trip.0 - trip.1 - trip.2) && is_valid_square(trip.0 + trip.2)
        && is_valid_square(trip.0 - trip.1 + trip.2) && is_valid_square(trip.0 + trip.1 - trip.2)
        && is_valid_square(trip.0 - trip.2) && is_valid_square(trip.0 + trip.1 + trip.2) && is_valid_square(trip.0 - trip.1)
}