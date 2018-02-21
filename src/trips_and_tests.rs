use simplelog::*;
use std::fs::File;
use std::sync::{Arc, Mutex, Barrier, mpsc};
use std::thread;

#[derive(Debug)]
pub enum TGError {
    FailedRetrieve,
    NotActive,
    EmptyReturn,
    CommError
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
        TGError::FailedRetrieve => info!("Failed to get data."),
        TGError::NotActive => error!("Attempted to use inactive triplet generator."),
        TGError::EmptyReturn => info!("Produced empty return vec."),
        TGError::CommError => info!("Communication between threads failed.")
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
            return Err(TGError::NotActive);
        }
        self.s_inst.send(TGInst::Pause).unwrap();
        Ok(true)
    }

    pub fn get_data(&self, req_len: usize) -> Result<Result<Vec<(u64, u64, u64)>, TGError>, TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        self.s_inst.send(TGInst::Get(req_len)).unwrap();
        let ret = self.r_data.recv();
        match ret {
            Ok(TGTReturn::Data(good_stuff)) => return Ok(good_stuff),
            Ok(TGTReturn::Done) => return Err(TGError::NotActive),
            Err(_) => return Err(TGError::FailedRetrieve),
        }
    }

    pub fn progress(&self) -> Result<(u64, u64, u64), TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        let get_prog = self.r_data.recv();
        match get_prog {
            Ok(TGTReturn::Data(Ok(trip))) => Ok(trip[0]),
            Ok(TGTReturn::Data(Err(tgerr))) => Err(tgerr),
            Ok(TGTReturn::Done) => Err(TGError::NotActive),
            Err(_) => Err(TGError::CommError)
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
                                loop {
                                    let instr = tgt.r_inst.recv();
                                    match instr {
                                        Ok(TGInst::Play) => break,
                                        Ok(_) => tgt.s_data.send(TGTReturn::Data(Err(TGError::NotActive))).unwrap(),
                                        Err(_) => log_tgerror(TGError::CommError)
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
                    if working && trips.len() < get_amt && all_valid((x, y, z)) {
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