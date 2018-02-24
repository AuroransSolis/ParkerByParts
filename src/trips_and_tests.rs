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
    At,
    BufferedAmt
}

#[derive(Debug)]
pub enum TGTReturn {
    Data(Result<Vec<(u64, u64, u64)>, TGError>),
    EmptyDone,
    Done(Result<Vec<(u64, u64, u64)>, TGError>),
    Info(Result<usize, TGError>)
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
            return Err(TGError::NotActive);
        }
        if self.paused {
            return Err(TGError::Paused);
        }
        if let Err(_) = self.s_inst.send(TGInst::Pause) {
            return Err(TGError::FailedSend);
        }
        match self.r_data.recv() {
            Ok(TGTReturn::EmptyDone) => {
                self.paused = true;
                Ok(true)
            },
            Ok(response) => unreachable!("Got unexpected response to pause request: {:?}", response),
            Err(_) => Err(TGError::FailedReceive)
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
            Ok(TGTReturn::EmptyDone) => {
                self.paused = false;
                Ok(true)
            },
            Ok(response) => unreachable!("Got unexpected response to play request: {:?}", response),
            Err(_) => Err(TGError::FailedReceive)
        }
    }

    pub fn get_data(&mut self, req_len: usize) -> Result<Vec<(u64, u64, u64)>, TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        if self.paused {
            return Err(TGError::Paused);
        }
        if let Err(_) = self.s_inst.send(TGInst::Get(req_len)) {
            return Err(TGError::FailedSend);
        }
        match self.r_data.recv() {
            Ok(TGTReturn::Data(Ok(good_stuff))) => Ok(good_stuff),
            Ok(TGTReturn::Data(Err(_))) => Err(TGError::EmptyReturn),
            Ok(TGTReturn::Done(Ok(good_stuff))) => {
                self.active = false;
                Ok(good_stuff)
            },
            Ok(response) => unreachable!("Got unexpected response to data request: {:?}", response),
            Err(_) => Err(TGError::FailedReceive)
        }
    }

    pub fn progress(&self) -> Result<(u64, u64, u64), TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        if let Err(_) = self.s_inst.send(TGInst::At) {
            return Err(TGError::FailedSend);
        }
        match self.r_data.recv() {
            Ok(TGTReturn::Data(Ok(trip))) => Ok(trip[0]),
            Ok(TGTReturn::Data(Err(tgerr))) => Err(tgerr),
            Ok(response) => unreachable!("Got unexpected response to progress query: {:?}", response),
            Err(_) => Err(TGError::FailedReceive)
        }
    }

    pub fn query_buffer_size(&mut self) -> Result<usize, TGError> {
        if !self.active {
            return Err(TGError::NotActive);
        }
        if let Err(_) = self.s_inst.send(TGInst::BufferedAmt) {
            return Err(TGError::FailedSend);
        }
        match self.r_data.recv() {
            Ok(TGTReturn::Info(Ok(amt))) => Ok(amt),
            Ok(response) => unreachable!("Received unexpected response to buffer size query: {:?}", response),
            Err(_) => Err(TGError::FailedReceive)
        }
    }
}

pub struct TripGenThread {
    r_inst: mpsc::Receiver<TGInst>,
    s_data: mpsc::Sender<TGTReturn>,
    max: u64,
    buf_size: usize
}

impl TripGenThread {
    pub fn new(inst_r: mpsc::Receiver<TGInst>, data_s: mpsc::Sender<TGTReturn>, max: u64, buf_size: usize) -> Self {
        TripGenThread {
            r_inst: inst_r,
            s_data: data_s,
            max: max,
            buf_size: buf_size,
        }
    }
}

pub fn run(tgt: TripGenThread) -> thread::JoinHandle<()> {
    let tgen = thread::spawn(move || {
        let mut at = (0, 0, 0);
        let mut buf: Vec<(u64, u64, u64)> = Vec::with_capacity(tgt.max as usize);
        let mut get_amt = 0;
        let mut working = false;
        for x in (0u64..).map(|x| x * x).take_while(|&x| x < tgt.max) {
            for y in 0..x {
                for z in 0..x - y {
                    /*while !working {
                        let inst = tgt.r_inst.recv();
                        match inst {
                            Ok(TGInst::Pause) => {
                                tgt.s_data.send(TGTReturn::EmptyDone).unwrap();
                                loop {
                                    let instr = tgt.r_inst.recv();
                                    match instr {
                                        Ok(TGInst::Play) => {
                                            tgt.s_data.send(TGTReturn::EmptyDone).unwrap();
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
                    if trips.len() < get_amt && x > 0 && y > 0 && z > 0 && y != z && all_valid((x, y, z)) {
                        trips.push((x, y, z));
                        if trips.len() == get_amt {
                            let last = trips[trips.len() - 1];
                            tgt.s_data.send(TGTReturn::Data(Ok(trips.clone()))).unwrap();
                            trips.clear();
                            get_amt = 0;
                            working = false;
                            at = ((last.0 as f64).sqrt() as u64, last.1, last.2);
                        }
                    }*/
                    if !working {
                        if buf.len() < tgt.buf_size {
                            match tgt.r_inst.try_recv() {
                                Ok(TGInst::Pause) => {
                                    tgt.s_data.send(TGTReturn::EmptyDone).unwrap();
                                    loop {
                                        let instr = tgt.r_inst.recv();
                                        match instr {
                                            Ok(TGInst::Play) => {
                                                tgt.s_data.send(TGTReturn::EmptyDone).unwrap();
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
                                Ok(TGInst::At) => tgt.s_data.send(TGTReturn::Data(Ok(vec![buf[buf.len() - 1]]))).unwrap(),
                                Ok(TGInst::BufferedAmt) => tgt.s_data.send(TGTReturn::Info(Ok(buf.len()))).unwrap(),
                                _ => {}
                            }
                            if x > 0 && y > 0 && z > 0 && y != z && all_valid((x, y, z)) {
                                buf.push((x, y, z));
                                continue;
                            }
                        }
                        while !working && buf.len() == tgt.buf_size {
                            //println!("TGen ==> Working: f | Buffer: full | Pausing to wait for instruction.");
                            match tgt.r_inst.recv() {
                                Ok(TGInst::Get(amt)) => {
                                    get_amt = amt;
                                    working = true;
                                },
                                Ok(TGInst::At) => tgt.s_data.send(TGTReturn::Data(Ok(vec![at.clone()]))).unwrap(),
                                Ok(TGInst::BufferedAmt) => tgt.s_data.send(TGTReturn::Info(Ok(buf.len()))).unwrap(),
                                _ => {}
                            }
                        }
                    }
                    if working && buf.len() > get_amt {
                        let mut ret: Vec<(u64, u64, u64)> = Vec::with_capacity(get_amt);
                        for _ in 0..get_amt {
                            ret.push(buf.swap_remove(0));
                            let end = buf.len() - 1;
                            buf.swap(0, end);
                        }
                        let last = ret[ret.len() - 1];
                        tgt.s_data.send(TGTReturn::Data(Ok(ret))).unwrap();
                        get_amt = 0;
                        working = false;
                        at = ((last.0 as f64).sqrt() as u64, last.1, last.2);
                        continue;
                    } else if working && buf.len() == get_amt {
                        tgt.s_data.send(TGTReturn::Data(Ok(buf.clone()))).unwrap();
                        let last = buf[buf.len() - 1];
                        buf.clear();
                        get_amt = 0;
                        working = false;
                        at = ((last.0 as f64).sqrt() as u64, last.1, last.2);
                        continue;
                    } else {
                        if x > 0 && y > 0 && z > 0 && y != z && all_valid((x, y, z)) {
                            buf.push((x, y, z));
                            continue;
                        }
                    }
                }
            }
        }
        if working && buf.len() > 0 && buf.len() < get_amt {
            tgt.s_data.send(TGTReturn::Done(Ok(buf.clone()))).unwrap();
        }
        if working && buf.len() == 0 {
            tgt.s_data.send(TGTReturn::Data(Err(TGError::EmptyReturn))).unwrap();
        }
        println!("Finished iterating.");
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