use std::ops::{Add, AddAssign, Deref, Mul};

use anyhow::{anyhow, Result};

pub struct Vector<T> {
    data: Vec<T>,
}

// pretend this is a heavy operation, CPU intensive
pub fn dot_product<T>(a: Vector<T>, b: Vector<T>) -> Result<T>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T>,
{
    if a.len() != b.len() {
        // a.len => a.data.len() (Deref trait)
        return Err(anyhow!("Vectors must be of the same length"));
    }

    let mut result = T::default();
    for i in 0..a.len() {
        result += a[i] * b[i];
    }
    Ok(result)
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Vector { data: data.into() }
    }

    // pub fn len(&self) -> usize {
    //     self.data.len()
    // }

    // pub fn is_empty(&self) -> bool {
    //     self.data.is_empty()
    // }

    // pub fn iter(&self) -> impl Iterator<Item = &T> {
    //     self.data.iter()
    // }
}

// impl<T> Index<usize> for Vector<T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.data[index]
//     }
// }
