fn calc_4x4() {
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

fn main() {
    calc_4x4()
}
