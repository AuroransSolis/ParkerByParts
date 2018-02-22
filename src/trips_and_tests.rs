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
    Paused,
    Why
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
        TGError::Paused => info!("Attempted to send instruction while thread is paused."),
        TGError::Why => info!("Attempted to resume a resumed thread. Nice.")
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
    s_inst: mpsc::Sender<TGInst>,
    r_data: mpsc::Receiver<TGTReturn>,
    pub at: (u64, u64, u64),
    pub active: bool,
    pub paused: bool
}

impl TripGenMain {
    pub fn new(inst_send: mpsc::Sender<TGInst>, recv_data: mpsc::Receiver<TGTReturn>) -> Self {
        TripGenMain {
            s_inst: inst_send,
            r_data: recv_data,
            at: (0, 0, 0),
            active: true,
            paused: false
        }
    }

    pub fn pause(&mut self) -> Result<bool, TGError> {
        if !self.active {
            println!("Attempted to pause inactive TGen.");
            return Err(TGError::NotActive);
        }
        if self.paused {
            println!("Attempted to pause paused TGen.");
            return Err(TGError::Paused);
        }
        println!("Sending pause command.");
        let res = self.s_inst.send(TGInst::Pause);
        match res {
            Ok(_) => {
                println!("Pause command sent successfully.");
            },
            Err(_) => {
                println!("Error sending pause command.");
                println!("==>Finished running pause instruction.");
                return Err(TGError::FailedSend);
            }
        }
        match self.r_data.recv() {
            Ok(TGTReturn::Done) => {
                self.paused = true;
                Ok(true)
            },
            Ok(TGTReturn::Data(_)) => unreachable!("Got data back on pause response."),
            Err(e) => Err(TGError::FailedReceive)}
        }
    }

    pub fn play(&mut self) -> Result<bool, TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        if !self.paused {
            return Err(TGError::Why);
        }
        if let Err(_) = self.s_inst.send(TGInst::Play) {
            return Err(TGError::FailedSend)
        }
        match self.r_data.recv() {
            Ok(TGTReturn::Done) => {
                self.paused = false;
                Ok(true)
            },
            Ok(TGTReturn::Data(_)) => unreachable!("Got data back on play response."),
            Err(e) => {
                Err(TGError::FailedReceive)
            }
        }
    }

    pub fn get_data(&mut self, req_len: usize) -> Result<Result<Vec<(u64, u64, u64)>, TGError>, TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        if self.paused {
            return Err(TGError::Paused);
        }
        if let Err(_) = self.s_inst.send(TGInst::Get(req_len)) {
            return Err(TGError::FailedSend)
        }
        match self.r_data.recv() {
            Ok(TGTReturn::Data(good_stuff)) => return Ok(good_stuff),
            Ok(TGTReturn::Done) => {
                self.active = false;
                return Err(TGError::NotActive)
            },
            Err(_) => return Err(TGError::FailedReceive),
        }
    }

    pub fn progress(&mut self) -> Result<(u64, u64, u64), TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        if self.paused {
            return Err(TGError::Paused);
        }
        if let Err(_) = self.s_inst.send(TGInst::At) {
            return Err(TGError::FailedSend);
        }
        match self.r_data.recv() {
            Ok(TGTReturn::Data(Ok(trip))) => Ok(trip[0]),
            Ok(TGTReturn::Data(Err(tgerr))) => Err(tgerr),
            Ok(TGTReturn::Done) => {
                self.active = false;
                Err(TGError::NotActive)
            },
            Err(_) => Err(TGError::FailedReceive)
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
        let bar = TG_BAR.clone();
        let mut at = (0, 0, 0);
        let mut trips: Vec<(u64, u64, u64)> = Vec::new();
        let mut get_amt = 0;
        let mut working = false;
        for x in (0u64..).map(|x| x * x).take_while(|x| x * x < tgt.max) {
            for y in 0..x {
                for z in 0..x - y {
                    while !working {
                        let inst = tgt.r_inst.recv();
                        match inst {
                            Ok(TGInst::Pause) => {
                                tgt.s_data.send(TGTReturn::Done).unwrap();
                                loop {
                                    let instr = tgt.r_inst.recv();
                                    match instr {
                                        Ok(TGInst::Play) => {
                                            tgt.s_data.send(TGTReturn::Done).unwrap();
                                            break;
                                        },
                                        Ok(_) => tgt.s_data.send(TGTReturn::Data(Err(TGError::Paused))).unwrap(),
                                        Err(_) => log_tgerror(TGError::FailedReceive)
                                    }
                                }
                            },
                            Ok(TGInst::Get(amt)) => {
                                get_amt = amt;
                                working = true;
                            },
                            Ok(TGInst::At) => {
                                tgt.s_data.send(TGTReturn::Data(Ok(vec![at.clone()]))).unwrap();
                            },
                            _ => {}
                        }
                    }
                    if working && trips.len() < get_amt && x > 0 && y > 0 && z > 0 && y != z && all_valid((x, y, z)) {
                        trips.push((x, y, z));
                    }
                    if working && trips.len() == get_amt {
                        let last = trips[trips.len() - 1];
                        tgt.s_data.send(TGTReturn::Data(Ok(trips.clone()))).unwrap();
                        trips.clear();
                        get_amt = 0;
                        working = false;
                        at = ((last.0 as f64).sqrt() as u64, last.1, last.2);
                    }
                }
            }
        }
        if working && trips.len() > 0 && trips.len() < get_amt {
            tgt.s_data.send(TGTReturn::Data(Ok(trips.clone()))).unwrap();
        }
        if working && trips.len() == 0 {
            tgt.s_data.send(TGTReturn::Data(Err(TGError::EmptyReturn))).unwrap();
        }
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