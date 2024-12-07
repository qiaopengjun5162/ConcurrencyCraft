use anyhow::Result;
use concurrency::Matrix;

fn main() -> Result<()> {
    let a = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
    let b = Matrix::new(3, 2, vec![1, 2, 3, 4, 5, 6]);
    let c = a * b;
    // println!("c = {:?}", c); // c = Matrix (rows: 2, cols: 2, data: {22 28, 49 64})
    println!("a * b = {}", c); // a * b = {22 28, 49 64}

    Ok(())
}
