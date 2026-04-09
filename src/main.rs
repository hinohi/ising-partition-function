use ising_partition_function::*;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get() as u64)
        .unwrap_or(12);

    // 6x6: verify transfer matrix against brute force
    println!("=== 6x6 verification ===");

    println!("Brute force ({threads} threads) ...");
    let t = Instant::now();
    let bf = calc_6x6(threads);
    println!("  {:.3}s", t.elapsed().as_secs_f64());

    println!("Transfer matrix ...");
    let t = Instant::now();
    let tm = transfer_matrix::calc_6x6_transfer();
    println!("  {:.3}s", t.elapsed().as_secs_f64());

    if bf == tm {
        println!("OK: results match");
    } else {
        eprintln!("ERROR: results do not match!");
        std::process::exit(1);
    }
    bf.save_file("data/6x6.txt")?;

    // 7x7
    println!("\n=== 7x7 ===");
    println!("Transfer matrix ...");
    let t = Instant::now();
    let n7 = transfer_matrix::calc_7x7_transfer();
    println!("  {:.3}s", t.elapsed().as_secs_f64());
    n7.save_file("data/7x7.txt")?;
    println!("{n7}");

    // 8x8
    println!("\n=== 8x8 ===");
    println!("Transfer matrix ...");
    let t = Instant::now();
    let n8 = transfer_matrix::calc_8x8_transfer();
    println!("  {:.3}s", t.elapsed().as_secs_f64());
    n8.save_file("data/8x8.txt")?;
    println!("{n8}");

    Ok(())
}
