use crate::rankers::{Ranker, RankingResult};
use std::collections::BinaryHeap;
use std::fmt::Display;

const DEFAULT_K1: f64 = 1.2;
const DEFAULT_B: f64 = 0.75;

pub struct BM25Ranker<T> {
    k1: f64,
    b: f64,
    splitter: Box<dyn Splitter<T>>,
}

#[derive(Default)]
pub struct BM25RankerBuilder<T>
where
    T: Display + Clone,
{
    k1: f64,
    b: f64,
    splitter: Option<Box<dyn Splitter<T>>>,
}

impl<T> BM25RankerBuilder<T>
where
    T: Display + Clone,
{
    pub fn new() -> BM25RankerBuilder<T> {
        BM25RankerBuilder {
            k1: DEFAULT_K1,
            b: DEFAULT_B,
            splitter: None,
        }
    }

    pub fn k1(mut self, k1: f64) -> BM25RankerBuilder<T> {
        self.k1 = k1;
        self
    }

    pub fn b(mut self, b: f64) -> BM25RankerBuilder<T> {
        self.b = b;
        self
    }

    pub fn build(self) -> BM25Ranker<T> {
        BM25Ranker {
            k1: self.k1,
            b: self.b,
            splitter: match self.splitter {
                Some(splitter) => splitter,
                None => Box::new(WhitespaceSplitter {}),
            },
        }
    }
}

impl<T> BM25Ranker<T>
where
    T: Display + Clone,
{
    pub fn new() -> BM25RankerBuilder<T> {
        BM25RankerBuilder::new()
    }

    fn doc_score(&self, doc: T, query: T, corpus: &Vec<T>, avg_doc_len: f64, k1: f64, b: f64) -> f64
    where
        T: Display + Clone,
    {
        let query_terms = self.splitter.split(query);
        let doc_terms = self.splitter.split(doc);
        let mut score = 0.0;
        for q_i in query_terms.iter() {
            let idf = idf(q_i.as_str(), corpus);
            let mut q_i_doc = 0;
            for term in doc_terms.iter() {
                if term == q_i {
                    q_i_doc += 1;
                }
            }
            let q_i_doc = q_i_doc as f64;
            let num = q_i_doc * (k1 + 1.0);
            let denom = q_i_doc + k1 * (1.0 - b + b * doc_terms.len() as f64 / avg_doc_len);

            score += idf * num / denom;
        }
        score
    }
}

pub trait Splitter<T>
where
    T: Display + Clone,
{
    fn split(&self, s: T) -> Vec<String>;
}

struct WhitespaceSplitter;

impl<T> Splitter<T> for WhitespaceSplitter
where
    T: Display + Clone,
{
    fn split(&self, s: T) -> Vec<String> {
        s.to_string()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
}

impl<T> Ranker<T> for BM25Ranker<T> {
    fn rank(
        &self,
        query: T,
        corpus: Vec<T>,
        max_num_docs: Option<usize>,
    ) -> Result<Vec<RankingResult<T>>, Box<dyn std::error::Error>>
    where
        T: Display + Clone,
    {
        let mut total_doc_len = 0;
        for doc in corpus.iter() {
            total_doc_len += self.splitter.split(doc.clone()).len();
        }
        let avg_doc_len = total_doc_len as f64 / corpus.len() as f64;
        let mut heap = BinaryHeap::<RankingResult<T>>::new();
        for doc in corpus.iter() {
            let score = self.doc_score(
                doc.clone(),
                query.clone(),
                &corpus,
                avg_doc_len,
                self.k1,
                self.b,
            );
            heap.push(RankingResult {
                score,
                doc: doc.clone(),
            });
        }
        let mut ranked_results = Vec::<RankingResult<T>>::new();
        match max_num_docs {
            Some(max_num_docs) => {
                for _ in 0..max_num_docs {
                    if let Some(result) = heap.pop() {
                        ranked_results.push(result);
                    }
                }
            }
            None => {
                while let Some(result) = heap.pop() {
                    ranked_results.push(result);
                }
            }
        };
        Ok(ranked_results)
    }
}

fn idf<T>(q_i: &str, corpus: &Vec<T>) -> f64
where
    T: Display + Clone,
{
    let n = corpus.len() as f64;
    let mut n_q = 0;
    for doc in corpus.iter() {
        if doc.to_string().contains(q_i) {
            n_q += 1;
        }
    }
    let n_q = n_q as f64;
    ((n - n_q + 0.5) / (n_q + 0.5) + 1.0).ln()
}
