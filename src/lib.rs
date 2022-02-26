#![allow(clippy::unusual_byte_groupings)]

fn print_number_of_state<const M: usize, const E: usize>(number_of_state: [[u64; M]; E]) {
    let mut width = [3; M];
    print!("  \\m");
    for (m, w) in width.iter_mut().enumerate() {
        let mag = (m as i32 * 2) - M as i32 + 1;
        for same_ene in number_of_state.iter() {
            *w = (*w).max(same_ene[m].to_string().len());
        }
        print!(" {:d$}", mag, d = *w);
    }
    println!();
    print!(" e \\");
    for &w in width.iter() {
        print!("{}", "-".repeat(w + 1));
    }
    println!();
    for (ene, same_ene) in number_of_state.iter().enumerate() {
        if ene % 2 == 1 {
            continue;
        }
        print!("{:3}|", ene as i32 * 2 - E as i32 + 1);
        for (c, &w) in same_ene.iter().zip(width.iter()) {
            print!(" {:w$}", c);
        }
        println!();
    }
}

pub fn calc_2x2() {
    let mut number_of_state = [[0u64; 5]; 9];
    for cell in 0u32..=(1 << 4) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 2 | cell << 2 & 0b11_00;
        let slide_x = cell >> 1 & 0b01_01 | cell << 1 & 0b10_10;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state[ene as usize][mag as usize] += 1;
    }
    print_number_of_state(number_of_state);
}

pub fn calc_3x3() {
    let mut number_of_state = [[0u64; 10]; 19];
    for cell in 0u32..=(1 << 9) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 3 | cell << 6 & 0b111_000_000;
        let slide_x = cell >> 1 & 0b011_011_011 | cell << 2 & 0b100_100_100;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state[ene as usize][mag as usize] += 1;
    }
    print_number_of_state(number_of_state);
}

pub fn calc_4x4() {
    let mut number_of_state = [[0u64; 17]; 33];
    for cell in 0u32..=(1 << 16) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 4 | cell << 12 & 0xf000;
        let slide_x = cell >> 1 & 0x7777 | cell << 3 & 0x8888;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state[ene as usize][mag as usize] += 1;
    }
    print_number_of_state(number_of_state);
}

pub fn calc_5x5() {
    let mut number_of_state = [[0u64; 26]; 51];
    for cell in 0u32..=(1 << 25) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 5 | cell << 20 & 0b11111 << 20;
        let slide_x = cell >> 1 & 0b01111_01111_01111_01111_01111
            | cell << 4 & 0b10000_10000_10000_10000_10000;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state[ene as usize][mag as usize] += 1;
    }
    print_number_of_state(number_of_state);
}

pub fn calc_6x6() {
    let mut number_of_state = [[0u64; 37]; 73];
    let mut it = 0;
    for cell in 0u64..=(1 << 36) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 6 | cell << 30 & 0b111111 << 30;
        let slide_x = cell >> 1 & 0b011111_011111_011111_011111_011111_011111
            | cell << 5 & 0b100000_100000_100000_100000_100000_100000;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state[ene as usize][mag as usize] += 1;
        if cell & 0x3fff_ffff == 0 {
            eprintln!("{}/64", it);
            it += 1;
        }
    }
    print_number_of_state(number_of_state);
}
