use ising_partition_function::*;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    // 7x7
    println!("=== 7x7 ===");
    println!("Transfer matrix ...");
    let t = Instant::now();
    let n7 = transfer_matrix::calc_7x7_transfer();
    println!("  {:.3}s", t.elapsed().as_secs_f64());
    n7.save_file("data/7x7.txt")?;

    Ok(())
}
