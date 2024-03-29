use crate::utils::split_on_punc_and_whitespace;

pub trait Splitter {
    fn split(&self, s: &str) -> Vec<String>;
}

pub struct WhitespaceSplitter;
pub struct CharSplitter;
pub struct PuncSplitter;

impl Splitter for WhitespaceSplitter {
    fn split(&self, s: &str) -> Vec<String> {
        s.split_whitespace().map(|s| s.to_string()).collect()
    }
}

impl Splitter for CharSplitter {
    fn split(&self, s: &str) -> Vec<String> {
        s.chars().map(|c| c.to_string()).collect()
    }
}

impl Splitter for PuncSplitter {
    fn split(&self, s: &str) -> Vec<String> {
        split_on_punc_and_whitespace(s)
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}
