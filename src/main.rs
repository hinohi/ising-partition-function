use ising_partition_function::*;

fn main() -> std::io::Result<()> {
    let n = calc_2x2();
    n.save_file("data/2x2.txt")?;
    println!("{}", n);

    let n = calc_3x3();
    n.save_file("data/3x3.txt")?;
    println!("{}", n);

    let n = calc_4x4();
    n.save_file("data/4x4.txt")?;
    println!("{}", n);

    let n = calc_5x5();
    n.save_file("data/5x5.txt")?;
    println!("{}", n);

    let n = calc_6x6(6);
    n.save_file("data/6x6.txt")?;
    println!("{}", n);
    Ok(())
}
