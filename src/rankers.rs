use std::fmt::Display;

pub trait Ranker<T, U> {
    fn rank(
        &self,
        query: T,
        corpus: Vec<U>,
    ) -> Result<Vec<RankingResult<U>>, Box<dyn std::error::Error>>
    where
        T: Display + Clone,
        U: Display + Clone;
}

pub struct RankingResult<T>
where
    T: Display + Clone,
{
    pub score: f64,
    pub item: T,
}

impl<T> PartialOrd for RankingResult<T>
where
    T: Display + Clone,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<T> Eq for RankingResult<T> where T: Display + Clone {}

impl<T> PartialEq for RankingResult<T>
where
    T: Display + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl<T> Ord for RankingResult<T>
where
    T: Display + Clone,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
