use std::ops::Deref;

pub struct Vector<T> {
    data: Vec<T>,
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
