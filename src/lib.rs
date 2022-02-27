#![allow(clippy::unusual_byte_groupings)]

use std::{fmt::Write as FmtWrite, fs::File, io::Write, path::Path, thread};

#[derive(Debug)]
pub struct NumberOfState<const M: usize, const E: usize> {
    data: [[u64; M]; E],
}

impl<const M: usize, const E: usize> NumberOfState<M, E> {
    pub fn new() -> NumberOfState<M, E> {
        NumberOfState { data: [[0; M]; E] }
    }

    #[inline]
    pub fn incr(&mut self, ene: u32, mag: u32) {
        self.data[ene as usize][mag as usize] += 1;
    }

    pub fn save_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<usize> {
        let mut n = 0;
        let mut file = File::create(path)?;
        for (ene, same_ene) in self.data.iter().enumerate() {
            let ene = ene as i32 * 2 - E as i32 + 1;
            for (m, &c) in same_ene.iter().enumerate() {
                let m = m as i32 * 2 - M as i32 + 1;
                if c != 0 {
                    writeln!(file, "{} {} {}", ene, m, c)?;
                    n += 1;
                }
            }
        }
        Ok(n)
    }
}

impl<const M: usize, const E: usize> Default for NumberOfState<M, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const M: usize, const E: usize> std::fmt::Display for NumberOfState<M, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut width = [3; M];
        f.write_str("  \\m")?;
        for (m, w) in width.iter_mut().enumerate() {
            let mag = m as i32 * 2 - M as i32 + 1;
            for same_ene in self.data.iter() {
                *w = (*w).max(same_ene[m].to_string().len());
            }
            f.write_str(&format!(" {:d$}", mag, d = *w))?;
        }
        f.write_str("\n e \\")?;
        for &w in width.iter() {
            f.write_str("-".repeat(w + 1).as_str())?;
        }
        f.write_char('\n')?;
        for (ene, same_ene) in self.data.iter().enumerate() {
            if ene % 2 == 1 {
                continue;
            }
            f.write_str(&format!("{:3}|", ene as i32 * 2 - E as i32 + 1))?;
            for (c, &w) in same_ene.iter().zip(width.iter()) {
                f.write_str(&format!(" {:w$}", c))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl<const M: usize, const E: usize> std::ops::AddAssign for NumberOfState<M, E> {
    fn add_assign(&mut self, rhs: Self) {
        for (a, b) in self.data.iter_mut().zip(rhs.data) {
            for (a, b) in a.iter_mut().zip(b) {
                *a += b;
            }
        }
    }
}

pub fn calc_2x2() -> NumberOfState<5, 9> {
    let mut number_of_state = NumberOfState::new();
    for cell in 0u32..=(1 << 4) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 2 | cell << 2 & 0b11_00;
        let slide_x = cell >> 1 & 0b01_01 | cell << 1 & 0b10_10;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state.incr(ene, mag);
    }
    number_of_state
}

pub fn calc_3x3() -> NumberOfState<10, 19> {
    let mut number_of_state = NumberOfState::new();
    for cell in 0u32..=(1 << 9) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 3 | cell << 6 & 0b111_000_000;
        let slide_x = cell >> 1 & 0b011_011_011 | cell << 2 & 0b100_100_100;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state.incr(ene, mag);
    }
    number_of_state
}

pub fn calc_4x4() -> NumberOfState<17, 33> {
    let mut number_of_state = NumberOfState::new();
    for cell in 0u32..=(1 << 16) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 4 | cell << 12 & 0xf000;
        let slide_x = cell >> 1 & 0x7777 | cell << 3 & 0x8888;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state.incr(ene, mag);
    }
    number_of_state
}

pub fn calc_5x5() -> NumberOfState<26, 51> {
    let mut number_of_state = NumberOfState::new();
    for cell in 0u32..=(1 << 25) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 5 | cell << 20 & 0b11111 << 20;
        let slide_x = cell >> 1 & 0b01111_01111_01111_01111_01111
            | cell << 4 & 0b10000_10000_10000_10000_10000;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state.incr(ene, mag);
    }
    number_of_state
}

pub fn calc_6x6(threads: u64) -> NumberOfState<37, 73> {
    fn calc(start: u64, end: u64) -> NumberOfState<37, 73> {
        let mut number_of_state = NumberOfState::new();
        for cell in start..end {
            let mag = cell.count_ones();
            let slide_y = cell >> 6 | cell << 30 & 0b111111 << 30;
            let slide_x = cell >> 1 & 0b011111_011111_011111_011111_011111_011111
                | cell << 5 & 0b100000_100000_100000_100000_100000_100000;
            let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
            number_of_state.incr(ene, mag);
        }
        number_of_state
    }

    let mut handles = Vec::new();
    let n = 1 << 36;
    for i in 0..threads - 1 {
        handles.push(thread::spawn(move || {
            calc(n / threads * i, n / threads * (i + 1))
        }));
    }
    let mut number_of_state = calc(n / threads * (threads - 1), n);
    for h in handles {
        number_of_state += h.join().unwrap();
    }
    number_of_state
}
