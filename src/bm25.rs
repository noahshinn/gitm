use crate::rankers::{Ranker, RankingResult};
use crate::retrievers::Retriever;
use crate::store::Store;
use std::collections::BinaryHeap;
use std::fmt::Display;

const DEFAULT_K1: f64 = 1.2;
const DEFAULT_B: f64 = 0.75;

pub struct BM25Ranker<'a> {
    k1: f64,
    b: f64,
    splitter: &'a dyn Splitter,
}

pub struct BM25RankerBuilder {
    k1: f64,
    b: f64,
    splitter: &'static dyn Splitter,
}

impl BM25RankerBuilder {
    pub fn new() -> BM25RankerBuilder {
        BM25RankerBuilder {
            k1: DEFAULT_K1,
            b: DEFAULT_B,
            splitter: &WhitespaceSplitter,
        }
    }

    pub fn k1(mut self, k1: f64) -> BM25RankerBuilder {
        self.k1 = k1;
        self
    }

    pub fn b(mut self, b: f64) -> BM25RankerBuilder {
        self.b = b;
        self
    }

    pub fn splitter(mut self, splitter: &'static dyn Splitter) -> BM25RankerBuilder {
        self.splitter = splitter;
        self
    }

    // include lifetime parameter to ensure that the Splitter is not dropped
    pub fn build(self) -> BM25Ranker<'static> {
        BM25Ranker {
            k1: self.k1,
            b: self.b,
            splitter: self.splitter,
        }
    }
}

impl BM25Ranker<'_> {
    pub fn new() -> BM25Ranker<'static> {
        BM25Ranker::builder().build()
    }

    pub fn builder() -> BM25RankerBuilder {
        BM25RankerBuilder::new()
    }
}

pub trait Splitter {
    fn split(&self, s: String) -> Vec<String>;
}

fn doc_score<T, U>(
    doc: U,
    query: T,
    corpus: &Vec<U>,
    splitter: &dyn Splitter,
    avg_doc_len: f64,
    k1: f64,
    b: f64,
) -> f64
where
    T: Display + Clone,
    U: Display + Clone,
{
    let query_terms = splitter.split(query.to_string());
    let doc_terms = splitter.split(doc.to_string());
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

struct WhitespaceSplitter;

impl Splitter for WhitespaceSplitter {
    fn split(&self, s: String) -> Vec<String> {
        s.to_string()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
}

struct CharSplitter;

impl Splitter for CharSplitter {
    fn split(&self, s: String) -> Vec<String> {
        s.to_string().chars().map(|c| c.to_string()).collect()
    }
}

impl<T, U> Ranker<T, U> for BM25Ranker<'_>
where
    U: Display + Clone,
    T: Display + Clone,
{
    fn rank(
        &self,
        query: T,
        corpus: Vec<U>,
    ) -> Result<Vec<RankingResult<U>>, Box<dyn std::error::Error>> {
        let mut total_doc_len = 0;
        for doc in corpus.iter() {
            total_doc_len += self.splitter.split(doc.to_string()).len();
        }
        let avg_doc_len = total_doc_len as f64 / corpus.len() as f64;
        let mut heap = BinaryHeap::<RankingResult<U>>::new();
        for doc in corpus.iter() {
            let score = doc_score(
                doc.clone(),
                query.clone(),
                &corpus,
                self.splitter,
                avg_doc_len,
                self.k1,
                self.b,
            );
            heap.push(RankingResult::<U> {
                score,
                item: doc.clone(),
            });
        }
        let mut ranked_results = Vec::<RankingResult<U>>::new();
        while let Some(result) = heap.pop() {
            ranked_results.push(result);
        }
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

pub struct BM25Retriever {
    ranker: BM25Ranker<'static>,
}

pub struct BM25RetrieverBuilder {
    ranker: Option<BM25Ranker<'static>>,
}

impl BM25Retriever {
    pub fn new() -> Self {
        Self {
            ranker: BM25Ranker::builder().build(),
        }
    }

    pub fn builder() -> BM25RetrieverBuilder {
        BM25RetrieverBuilder::new()
    }
}

impl BM25RetrieverBuilder {
    pub fn new() -> Self {
        Self { ranker: None }
    }

    pub fn ranker(mut self, ranker: BM25Ranker<'static>) -> Self {
        self.ranker = Some(ranker);
        self
    }

    pub fn build(self) -> BM25Retriever {
        BM25Retriever {
            ranker: match self.ranker {
                Some(ranker) => ranker,
                None => BM25Ranker::builder().build(),
            },
        }
    }
}

impl<T, U> Retriever<T, U> for BM25Retriever
where
    T: Display + Clone,
    U: Display + Clone,
{
    fn retrieve(
        &self,
        query: T,
        store: Store<U>,
        max_num_results: usize,
    ) -> Result<Vec<U>, Box<dyn std::error::Error>> {
        let ranked_results = self.ranker.rank(query, store.data)?;
        let mut results = Vec::new();
        for result in ranked_results {
            if results.len() >= max_num_results {
                break;
            }
            results.push(result.item);
        }
        Ok(results)
    }
}
