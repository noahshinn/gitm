use crate::store::Store;
use std::fmt::Display;

pub trait Retriever<T>
where
    T: Display + Clone,
{
    fn retrieve(
        &self,
        query: T,
        store: Store<T>,
        max_num_results: usize,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>>;
}
