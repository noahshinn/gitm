use crate::store::Store;
use std::fmt::Display;

pub trait Retriever<T, U>
where
    T: Display + Clone,
    U: Display + Clone,
{
    fn retrieve(
        &self,
        query: T,
        store: Store<U>,
        max_num_results: usize,
    ) -> Result<Vec<U>, Box<dyn std::error::Error>>;
}
