use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use anyhow::{anyhow, Result};
use oneshot::Sender;

use crate::{dot_product, Vector};

const NUM_THREADS: usize = 4;

/// 矩阵乘法的规则是：两个矩阵相乘时，第一个矩阵的列数必须等于第二个矩阵的行数。
/// 然后，通过将第一个矩阵的每一行与第二个矩阵的每一列相乘并求和，得到结果矩阵的每一个元素。
// [[1,2], [1,2], [1,2]] => [1, 2, 1, 2, 1, 2]
pub struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

pub struct MsgOutput<T> {
    idx: usize,
    value: T,
}

pub struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    // 只有当矩阵a的列数等于矩阵b的行数时，两个矩阵才能相乘
    if a.cols != b.rows {
        return Err(anyhow!("Matrix dimensions do not match"));
    }

    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Error sending result: {}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });

            tx
        })
        .collect::<Vec<_>>();
    // generate 4 threads which receive msg and do dot product

    let matrix_len = a.rows * b.cols;
    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);

    // 结果矩阵 data 的元素 data[i * b.cols + j] 是通过将矩阵 a 的第 i 行与矩阵 b 的第 j 列对应元素相乘并求和得到的。
    // 具体来说，a.data[i * a.cols + k] 表示矩阵 a 的第 i 行第 k 列的元素。
    // b.data[k * b.cols + j] 表示矩阵 b 的第 k 行第 j 列的元素。
    // 这两个元素相乘的结果累加到 data[i * b.cols + j] 中。
    // map reduce: map phase
    for i in 0..a.rows {
        for j in 0..b.cols {
            // for k in 0..a.cols {
            //     data[i * b.cols + j] += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            // }

            // 创建矩阵a的第i行和矩阵b的第j列的向量
            let row = Vector::new(&a.data[i * a.cols..(i + 1) * a.cols]);
            let col = Vector::new(
                b.data[j..]
                    .iter()
                    .step_by(b.cols)
                    .copied()
                    .collect::<Vec<_>>(),
            );

            let idx = i * b.cols + j;
            let input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input, tx);
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprintln!("Error sending message: {}", e);
            }
            receivers.push(rx);

            // 使用dot_product函数计算点积，并赋值给结果矩阵的对应元素
            // data[i * b.cols + j] = dot_product(a_row, b_col)?;
        }
    }

    // map reduce: reduce phase
    for receiver in receivers {
        let result = receiver.recv()?;
        data[result.idx] = result.value;
    }

    Ok(Matrix {
        rows: a.rows,
        cols: b.cols,
        data,
    })
}

impl<T: Debug> Matrix<T> {
    pub fn new(rows: usize, cols: usize, data: impl Into<Vec<T>>) -> Self {
        Self {
            rows,
            cols,
            data: data.into(),
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    // display a 2x3 as {1 2 3, 4 5 6}, 3x2 as {1 2, 3 4, 5 6}
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.rows {
            for j in 0..self.cols {
                write!(f, "{}", self.data[i * self.cols + j])?;
                if j != self.cols - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.rows - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Matrix (rows: {}, cols: {}, data: {})",
            self.rows, self.cols, self
        )
    }
}

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("Matrix multiply error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 矩阵 C 的计算过程如下：

    // C[0][0] = A[0][0] * B[0][0] + A[0][1] * B[1][0] + A[0][2] * B[2][0] = 1 * 1 + 2 * 3 + 3 * 5 = 22
    // C[0][1] = A[0][0] * B[0][1] + A[0][1] * B[1][1] + A[0][2] * B[2][1] = 1 * 2 + 2 * 4 + 3 * 6 = 28
    // C[1][0] = A[1][0] * B[0][0] + A[1][1] * B[1][0] + A[1][2] * B[2][0] = 4 * 1 + 5 * 3 + 6 * 5 = 49
    // C[1][1] = A[1][0] * B[0][1] + A[1][1] * B[1][1] + A[1][2] * B[2][1] = 4 * 2 + 5 * 4 + 6 * 6 = 64
    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new(2, 3, [1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(3, 2, [1, 2, 3, 4, 5, 6]);
        // let c = multiply(&a, &b)?;
        let c = a * b;
        assert_eq!(c.rows, 2);
        assert_eq!(c.cols, 2);
        assert_eq!(c.data, &[22, 28, 49, 64]);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(
            format!("{:?}", c),
            "Matrix (rows: 2, cols: 2, data: {22 28, 49 64})"
        );
        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new(2, 3, [1, 2, 3, 4, 5, 6]);
        assert_eq!(
            format!("{:?}", a),
            "Matrix (rows: 2, cols: 3, data: {1 2 3, 4 5 6})"
        );
        Ok(())
    }

    /// 矩阵乘法的规则是：结果矩阵的每个元素是对应行和列的元素相乘然后求和。所以，结果矩阵 c 的计算如下：
    /// c[0][0] = a[0][0] * b[0][0] + a[0][1] * b[1][0] = 1 * 1 + 2 * 3 = 1 + 6 = 7
    /// c[0][1] = a[0][0] * b[0][1] + a[0][1] * b[1][1] = 1 * 2 + 2 * 4 = 2 + 8 = 10
    /// c[1][0] = a[1][0] * b[0][0] + a[1][1] * b[1][0] = 3 * 1 + 4 * 3 = 3 + 12 = 15
    /// c[1][1] = a[1][0] * b[0][1] + a[1][1] * b[1][1] = 3 * 2 + 4 * 4 = 6 + 16 = 22
    #[test]
    fn test_matrix_display2() -> Result<()> {
        let a = Matrix::new(2, 2, [1, 2, 3, 4]);
        let b = Matrix::new(2, 2, [1, 2, 3, 4]);
        // let c = multiply(&a, &b)?;
        let c = a * b;
        assert_eq!(c.rows, 2);
        assert_eq!(c.cols, 2);
        assert_eq!(c.data, &[7, 10, 15, 22]);
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(
            format!("{:?}", c),
            "Matrix (rows: 2, cols: 2, data: {7 10, 15 22})"
        );
        Ok(())
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new(2, 3, [1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(2, 2, [1, 2, 3, 4]);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new(2, 3, [1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(2, 2, [1, 2, 3, 4]);

        let _ = a * b;
    }
}
