use std::cmp;

pub fn aks(num: u64) -> bool {
    if num == 2 {
        return true;
    }
    if num < 2 {
        return false;
    }
    if is_perf_pow(num) {
        return false;
    }
    let b2_log_n_sq = ((num as f64).log2() as u64).pow(2u32);
    let r = (2u64..).filter(|&n| get_mult_order(num, n) > b2_log_n_sq).next().unwrap() as u64;
    for n in 2..cmp::min(r, num - 1) {
        if num % n == 0 {
            return false;
        }
    }
    if num <= r {
        return true;
    }
    last_step(num, ((totient(num) as f64).sqrt() * (num as f64).log2()) as u64)
}

fn fast_exp(mut num: u64, mut pow: u16) -> u64 {
    let mut ret = 1;
    while pow > 0 {
        if pow & 1 == 1 {
            ret *= num;
        }
        num *= num;
        pow >>= 1;
    }
    ret
}

fn is_perf_pow(num: u64) -> bool {
    if num & (num - 1) == 0 {
        return true;
    }
    let n = (0..).filter(|n| (num << n) & 0x8000000000000000 == 0x8000000000000000).map(|n| 64 - n).next().unwrap();
    for m in 2..n {
        let mut low_a = 1u64;
        let mut high_a = 1u64 << (n / m + 1);
        while low_a < high_a - 1 {
            let mid_a = (low_a + high_a) >> 1;
            let ab = fast_exp(mid_a, m);
            if ab > num {
                high_a = mid_a;
            } else if ab < num {
                low_a = mid_a;
            } else {
                return true;
            }
        }
    }
    return false;
}

fn get_mult_order(num: u64, base: u64) -> u64 {
    (1u32..).filter(|&n| num.pow(n) % base == 1).next().unwrap() as u64
}

fn get_gcd(mut a: u64, mut b: u64) -> u64 {
    loop {
        if b == 0 {
            return a;
        } else {
            let c = b;
            b = a % b;
            a = c;
        }
    }
}

fn totient(num: u64) -> u64 {
    (1..num).filter(|&n| get_gcd(num, n) == 1).count() as u64
}

fn mod_fact(num: u64, m: u64) -> u64 {
    (1..num + 1).fold(1, |res, n: u64| res * n % m)
}

fn part_mod_fact(s: u64, e: u64, m: u64) -> u64 {
    (s..e + 1).fold(1, |res, n: u64| res * n % m)
}

fn last_step(num: u64, last: u64) -> bool {
    for n in 1..last {
        if part_mod_fact(num - n + 1, num, num) / mod_fact(n, num) % num != 0 {
            return false;
        }
    }
    true
}