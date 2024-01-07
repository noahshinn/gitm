use std::fmt::Display;
use std::sync::Mutex;

pub struct Store<T>
where
    T: Display + Clone,
{
    mu: Mutex<()>,
    pub data: Vec<T>,
}

impl<T> Store<T>
where
    T: Display + Clone,
{
    pub fn get(&self, index: usize) -> Option<&T> {
        let _ = self.mu.lock();
        self.data.get(index)
    }

    pub fn len(&self) -> usize {
        let _ = self.mu.lock();
        self.data.len()
    }
}

impl<T> From<Vec<T>> for Store<T>
where
    T: Display + Clone,
{
    fn from(data: Vec<T>) -> Self {
        Self {
            mu: Mutex::new(()),
            data,
        }
    }
}
