use crate::{dot_product, Vector};
use anyhow::Result;
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

// 根据矩阵大小动态计算线程数
fn calculate_thread_count(rows: usize, cols: usize) -> usize {
    let total_elements = rows * cols;
    if total_elements < 100 {
        1
    } else if total_elements < 1000 {
        2
    } else {
        4
    }
}

pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, rows: usize, cols: usize) -> Self {
        Self {
            data: data.into(),
            rows,
            cols,
        }
    }
}

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // display a 2x3 as {1 2 3, 4 5 6}, a 3x2 as {1 2, 3 4, 5 6}
        write!(f, "{{")?;
        for i in 0..self.rows {
            if i > 0 {
                write!(f, ", ")?;
            }
            for j in 0..self.cols {
                if j > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", self.data[i * self.cols + j])?;
            }
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Matrix {{ rows: {}, cols: {}, data: {} }}",
            self.rows, self.cols, self
        )
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Default + Send + Sync + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("Matrix multiplication failed")
    }
}
#[allow(dead_code)]
struct MatrixMultiplicationTask<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
    sender: oneshot::Sender<Result<T>>,
}

impl<T> MatrixMultiplicationTask<T> {
    fn new(idx: usize, row: Vector<T>, col: Vector<T>, sender: oneshot::Sender<Result<T>>) -> Self {
        Self {
            idx,
            row,
            col,
            sender,
        }
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Default + Send + Sync + 'static,
{
    if a.cols != b.rows {
        return Err(anyhow::anyhow!("Matrix dimensions do not match"));
    }

    let thread_count = calculate_thread_count(a.rows, b.cols);
    let senders = (0..thread_count)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<MatrixMultiplicationTask<T>>();

            thread::spawn(move || {
                for task in rx {
                    let result = dot_product(task.row, task.col);
                    if let Err(e) = task.sender.send(result) {
                        eprintln!("Failed to send matrix multiplication result: {}", e);
                    }
                }
            });

            tx
        })
        .collect::<Vec<_>>();

    let mut data = vec![T::default(); a.rows * b.cols];
    let mut receivers = Vec::with_capacity(a.rows * b.cols);

    for i in 0..a.rows {
        for j in 0..b.cols {
            let a_row = Vector::new(a.data[i * a.cols..(i + 1) * a.cols].to_vec());
            // 使用迭代器直接构建列向量，避免中间 Vec 分配
            let b_col = Vector::new(
                b.data
                    .iter()
                    .skip(j)
                    .step_by(b.cols)
                    .take(b.rows)
                    .cloned()
                    .collect(),
            );

            let (tx, rx) = oneshot::channel();
            let task = MatrixMultiplicationTask::new(i * b.cols + j, a_row, b_col, tx);
            senders[i % thread_count].send(task)?;
            receivers.push((i * b.cols + j, rx));
        }
    }

    // 收集结果并处理错误
    for (idx, receiver) in receivers {
        match receiver.recv()? {
            Ok(value) => data[idx] = value,
            Err(e) => return Err(anyhow::anyhow!("Error in matrix multiplication: {}", e)),
        }
    }

    Ok(Matrix {
        data,
        rows: a.rows,
        cols: b.cols,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_debug() -> Result<()> {
        let a = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new(vec![1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(
            format!("{:?}", c),
            "Matrix { rows: 2, cols: 2, data: {22 28, 49 64} }"
        );
        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], 3, 4);
        let b = Matrix::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], 4, 3);
        let c = a * b;
        assert_eq!(format!("{}", c), "{70 80 90, 158 184 210, 246 288 330}");
        Ok(())
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new(vec![1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new(vec![1, 2, 3, 4], 2, 2);
        let _ = a * b;
    }
}
