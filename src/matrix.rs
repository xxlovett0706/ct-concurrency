use anyhow::Result;
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
};

pub struct Matrix<T: fmt::Debug> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T: fmt::Debug> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, rows: usize, cols: usize) -> Self {
        Self {
            data: data.into(),
            rows,
            cols,
        }
    }
}

impl<T: fmt::Debug> fmt::Display for Matrix<T> {
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
                write!(f, "{:?}", self.data[i * self.cols + j])?;
            }
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl<T: fmt::Debug> fmt::Debug for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Matrix {{ rows: {}, cols: {}, data: {} }}",
            self.rows, self.cols, self
        )
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: fmt::Debug + Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Default,
{
    if a.cols != b.rows {
        return Err(anyhow::anyhow!("Matrix dimensions do not match"));
    }

    let mut data = vec![T::default(); a.rows * b.cols];

    for i in 0..a.rows {
        for j in 0..b.cols {
            let mut sum = T::default();
            for k in 0..a.cols {
                sum += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
            data[i * b.cols + j] = sum;
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
        let c = multiply(&a, &b)?;
        assert_eq!(c.rows, 2);
        assert_eq!(c.cols, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(
            format!("{:?}", c),
            "Matrix { rows: 2, cols: 2, data: {22 28, 49 64} }"
        );
        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new(vec![1, 2, 3, 4], 2, 2);
        let b = Matrix::new(vec![1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b)?;
        assert_eq!(format!("{}", c), "{7 10, 15 22}");
        Ok(())
    }
}
