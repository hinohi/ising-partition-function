use ising_partition_function::*;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    println!("=== 8x8 ===");
    println!("Transfer matrix ...");
    let t = Instant::now();
    let n8 = transfer_matrix::calc_8x8_transfer();
    println!("  {:.3}s", t.elapsed().as_secs_f64());
    n8.save_file("data/8x8.txt")?;

    Ok(())
}
