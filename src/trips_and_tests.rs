use simplelog::*;
use std::fs::File;
use std::sync::{Arc, Mutex, Barrier, mpsc};
use std::thread;

#[derive(Debug)]
pub enum TGError {
    FailedReceive,
    FailedSend,
    NotActive,
    EmptyReturn,
    Paused
}

pub fn tg_log_init() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("magic_search.log").unwrap())
        ]
    ).unwrap();
}

pub fn log_tgerror(error: TGError) {
    match error {
        TGError::FailedReceive => info!("Failed to receive data."),
        TGError::FailedSend => info!("Failed to send data."),
        TGError::NotActive => error!("Attempted to use inactive triplet generator."),
        TGError::EmptyReturn => info!("Produced empty return vec."),
        TGError::Paused => info!("Attempted to send instruction while thread is paused.")
    }
}

pub enum TGInst {
    Pause,
    Play,
    Get(usize),
    At
}

pub enum TGTReturn {
    Data(Result<Vec<(u64, u64, u64)>, TGError>),
    Done
}

pub struct TripGenMain {
    pub s_inst: mpsc::Sender<TGInst>,
    pub r_data: mpsc::Receiver<TGTReturn>,
    pub at: (u64, u64, u64),
    pub active: bool
}

impl TripGenMain {
    pub fn new(inst_send: mpsc::Sender<TGInst>, recv_data: mpsc::Receiver<TGTReturn>) -> Self {
        TripGenMain {
            s_inst: inst_send,
            r_data: recv_data,
            at: (0, 0, 0),
            active: true
        }
    }

    pub fn pause(&self) -> Result<bool, TGError> {
        if !self.active {
            println!("Attempted to pause inactive TGen.");
            return Err(TGError::NotActive);
        }
        println!("Sending pause command.");
        match self.s_inst.send(TGInst::Pause) {
            Ok(_) => {
                println!("Pause command sent successfully.");
                Ok(true)
            },
            Err(_) => {
                println!("Error sending pause command.");
                Err(TGError::FailedSend)
            }
        }
    }

    pub fn get_data(&mut self, req_len: usize) -> Result<Result<Vec<(u64, u64, u64)>, TGError>, TGError> {
        if !self.active {
            println!("Attempted to get data from inactive TGen.");
            return Err(TGError::NotActive);
        }
        match self.s_inst.send(TGInst::Get(req_len)) {
            Ok(_) => println!("Sent data request successfully."),
            Err(_) => {
                println!("Error sending data request.");
                return Err(TGError::FailedSend);
            }
        }
        println!("Attempting to receive data on data channel.");
        let ret = self.r_data.recv();
        println!("Received data on the data channel.");
        match ret {
            Ok(TGTReturn::Data(good_stuff)) => {
                println!("Got data back from TGen thread.");
                return Ok(good_stuff)
            },
            Ok(TGTReturn::Done) => {
                println!("TGen thread is done.");
                self.active = false;
                return Err(TGError::NotActive)
            },
            Err(_) => {
                println!("Failed to retrieve data.");
                return Err(TGError::FailedReceive)
            },
        }
    }

    pub fn progress(&mut self) -> Result<(u64, u64, u64), TGError> {
        if !self.active {
            println!("Attempted to get progress from inactive TGen.");
            return Err(TGError::NotActive);
        }
        println!("Attempting to get progress.");
        let get_prog = self.r_data.recv();
        match get_prog {
            Ok(TGTReturn::Data(Ok(trip))) => {
                println!("Got progress.");
                Ok(trip[0])
            },
            Ok(TGTReturn::Data(Err(tgerr))) => {
                println!("Got TGError.");
                Err(tgerr)
            },
            Ok(TGTReturn::Done) => {
                println!("Thread is done.");
                self.active = false;
                Err(TGError::NotActive)
            },
            Err(_) => {
                println!("Failed to receive progress.");
                Err(TGError::FailedReceive)
            }
        }
    }
}

lazy_static! {
    pub static ref TG_BAR: Arc<Barrier> = Arc::new(Barrier::new(2));
}

pub struct TripGenThread {
    r_inst: mpsc::Receiver<TGInst>,
    s_data: mpsc::Sender<TGTReturn>,
    max: u64,
}

impl TripGenThread {
    pub fn new(inst_r: mpsc::Receiver<TGInst>, data_s: mpsc::Sender<TGTReturn>, max: u64) -> Self {
        TripGenThread {
            r_inst: inst_r,
            s_data: data_s,
            max: max
        }
    }
}

pub fn run(tgt: TripGenThread) -> thread::JoinHandle<()> {
    let tgen = thread::spawn(move || {
        println!("Spawned TGen thread.");
        let bar = TG_BAR.clone();
        println!("Cloned barrier.");
        let mut at = (0, 0, 0);
        println!("Set at.");
        let mut trips: Vec<(u64, u64, u64)> = Vec::new();
        println!("Initialized triplets vec.");
        let mut get_amt = 0;
        let mut working = false;
        for x in (0u64..).map(|x| x * x).take_while(|x| x * x < tgt.max) {
            for y in 0..x {
                for z in 0..x - y {
                    while !working {
                        println!("TGen thread: calling .recv()");
                        let inst = tgt.r_inst.recv();
                        println!("Got message!");
                        match inst {
                            Ok(TGInst::Pause) => {
                                println!("Got pause request!");
                                loop {
                                    println!("Waiting for play instruction.");
                                    let instr = tgt.r_inst.recv();
                                    println!("Got message.");
                                    match instr {
                                        Ok(TGInst::Play) => {
                                            println!("Message was play.");
                                            break;
                                        },
                                        Ok(_) => {
                                            println!("Message was not play.");
                                            tgt.s_data.send(TGTReturn::Data(Err(TGError::Paused))).unwrap();
                                        },
                                        Err(_) => {
                                            println!("Got error.");
                                            log_tgerror(TGError::FailedReceive);
                                        }
                                    }
                                }
                            },
                            Ok(TGInst::Get(amt)) => {
                                println!("Got data request! Request length: {}", amt);
                                get_amt = amt;
                                working = true;
                                println!("Set get_amt and working.");
                            },
                            Ok(TGInst::At) => {
                                println!("Got at request.");
                                tgt.s_data.send(TGTReturn::Data(Ok(vec![at.clone()]))).unwrap();
                                println!("Responded to at request.");
                            },
                            _ => {}
                        }
                    }
                    if working && trips.len() < get_amt && all_valid((x, y, z)) {
                        trips.push((x, y, z));
                    }
                    if working && trips.len() == get_amt {
                        println!("Collected requested amount of data points ({})", get_amt);
                        let last = trips[trips.len() - 1];
                        println!("Sending requested data.");
                        tgt.s_data.send(TGTReturn::Data(Ok(trips.clone()))).unwrap();
                        println!("Sent requested data.");
                        trips.clear();
                        println!("Cleared data vec.");
                        get_amt = 0;
                        working = false;
                        println!("Reset get_amt and working.");
                        at = ((last.0 as f64).sqrt() as u64, last.1, last.2);
                        println!("Set at.");
                    }
                }
            }
        }
        if working && trips.len() > 0 && trips.len() < get_amt {
            println!("Iterators finished before the amount of requested data could be produced.");
            tgt.s_data.send(TGTReturn::Data(Ok(trips.clone()))).unwrap();
            println!("Sent gathered data.");
        }
        if working && trips.len() == 0 {
            println!("Got data request, but there were no valid triplets left to send for testing.");
            tgt.s_data.send(TGTReturn::Data(Err(TGError::EmptyReturn))).unwrap();
            println!("Sent empty return error.");
        }
        println!("Thread finished execution.");
    });
    tgen
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