use ising_partition_function::*;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get() as u64)
        .unwrap_or(12);

    println!("6x6 Ising model partition function");
    println!();

    // CPU
    println!("CPU ({threads} threads) ...");
    let t = Instant::now();
    let cpu = calc_6x6(threads);
    let cpu_time = t.elapsed();
    println!("  {:.3}s", cpu_time.as_secs_f64());
    cpu.save_file("data/6x6_cpu.txt")?;

    // GPU
    println!("GPU ...");
    let t = Instant::now();
    let gpu_result = gpu::calc_6x6_gpu();
    let gpu_time = t.elapsed();
    println!("  {:.3}s", gpu_time.as_secs_f64());
    gpu_result.save_file("data/6x6_gpu.txt")?;

    // Compare
    if cpu == gpu_result {
        println!("\nResults match");
    } else {
        println!("\nResults DO NOT match!");
    }
    println!(
        "Speedup: {:.2}x",
        cpu_time.as_secs_f64() / gpu_time.as_secs_f64()
    );

    Ok(())
}
