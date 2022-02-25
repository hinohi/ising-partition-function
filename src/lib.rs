pub fn calc_4x4() {
    let mut number_of_state = [[0; 17]; 33];
    for cell in 0u32..=(1 << 16) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 4 | cell << 12 & 0xf000;
        let slide_x = cell >> 1 & 0x7777 | cell << 3 & 0x8888;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state[ene as usize][mag as usize] += 1;
    }

    print!("  \\m");
    for m in -8..=8 {
        print!(" {:4}", m * 2);
    }
    println!();
    print!(" e \\");
    for _ in -8..=8 {
        print!("-----");
    }
    println!();
    for (ene, same_ene) in number_of_state.iter().enumerate() {
        if ene % 2 == 1 {
            continue;
        }
        print!("{:3}|", ene as i32 * 2 - 32);
        for c in same_ene.iter() {
            print!(" {:4}", c);
        }
        println!();
    }
}

pub fn calc_5x5() {
    let mut number_of_state = [[0u32; 26]; 51];
    for cell in 0u32..=(1 << 25) - 1 {
        let mag = cell.count_ones();
        let slide_y = cell >> 5 | cell << 20 & 0b11111 << 20;
        let slide_x = cell >> 1 & 0b01111_01111_01111_01111_01111
            | cell << 4 & 0b10000_10000_10000_10000_10000;
        let ene = (cell ^ slide_x).count_ones() + (cell ^ slide_y).count_ones();
        number_of_state[ene as usize][mag as usize] += 1;
    }

    let mut width = [0; 26];
    print!("  \\m");
    for (m, w) in width.iter_mut().enumerate() {
        let mag = (m as i32 * 2) - 25;
        *w = mag.to_string().len();
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
        print!("{:3}|", ene as i32 * 2 - 50);
        for (c, &w) in same_ene.iter().zip(width.iter()) {
            print!(" {:w$}", c);
        }
        println!();
    }
}

pub fn calc_6x6() {
    let mut number_of_state = [[0u32; 37]; 73];
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

    let mut width = [0; 37];
    print!("  \\m");
    for (m, w) in width.iter_mut().enumerate() {
        let mag = (m as i32 * 2) - 36;
        *w = mag.to_string().len();
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
        print!("{:3}|", ene as i32 * 2 - 72);
        for (c, &w) in same_ene.iter().zip(width.iter()) {
            print!(" {:w$}", c);
        }
        println!();
    }
}
