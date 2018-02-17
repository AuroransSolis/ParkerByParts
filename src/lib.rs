pub struct TripFetcher {
    pub max: u64,
    pub start: (usize, usize, usize),
    pub gucci: bool
}

impl TripFetcher {
    pub fn new(max: u64, start: (usize, usize, usize)) -> Self {
        TripFetcher {
            max: max,
            start: start,
            gucci: true
        }
    }

    //Get triplet vec
    pub fn get_triplets_vec(&mut self, vec_len: usize) -> Option<Vec<(u64, u64, u64)>> {
        let max = self.max;
        let start_trips = self.start;
        // Return None if any of the start conditions are too high
        if start_trips.0 > max as usize || start_trips.1 > max as usize || start_trips.2 > max as usize {
            self.gucci = false;
            println!("Triggered first None return.");
            None
        } else if !self.gucci {
            println!("Triggered second None return.");
            None
        } else {
            // Make vec to be returned
            let mut new_trips = Vec::new();
            let mut y_skip = start_trips.1;
            let mut z_skip = start_trips.2;
            let mut x_skip = 0;
            let mut iter = true;
            for x_num in (0u64..).skip(start_trips.0 as usize).take_while(|x| x * x < max) {
                if !iter {
                    break;
                }
                let x = x_num * x_num;
                x_skip = x_num;
                for y in (0..x).skip(y_skip as usize) {
                    if !iter {
                        break;
                    }
                    for z in (0..x - y).skip(z_skip as usize) {
                        //println!("x_num: {} | x: {}, y: {}, z: {} | Return vec len: {}", x_num, x, y, z, new_trips.len());
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
                println!("Triggered third None return.");
                return None;
            }
            // Deactivate fetcher if the end of the requested range is out of the range of max
            let starts = vec![x_skip as usize, start_trips.1, start_trips.2];
            if new_trips.len() != vec_len {
                println!("Deactivating fetcher.");
                self.gucci = false;
            }
            // Set new last element
            let last_elem = new_trips[new_trips.len() - 1];
            self.start = (x_skip as usize, last_elem.1 as usize, (last_elem.2 + 1) as usize);
            //println!("New start: {:?}", self.start);
            Some(new_trips)
        }
    }
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