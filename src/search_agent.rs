use crate::bm25::BM25Retriever;
use crate::classifier::{BinaryClassificationResult, BinaryClassifier};
use crate::git;
use crate::git::{Author, Commit};
use crate::github;
use crate::github::Issue;
use crate::llm::ChatModel;
use crate::mention_classifiers::{AuthorMentionBinaryClassifier, DateTimeMentionClassifier};
use crate::retrievers::Retriever;
use crate::store::Store;
use chrono::{DateTime, Local};

pub struct SearchAgent {
    git_client: git::Client,
    github_client: github::Client,
    model: ChatModel,
    author_mention_classifier: AuthorMentionBinaryClassifier,
    datetime_mention_classifier: DateTimeMentionClassifier,
}

pub enum SearchMode {
    Commits,
    Issues,
    CommitsAndIssues,
}

impl PartialEq for SearchMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SearchMode::Commits, SearchMode::Commits) => true,
            (SearchMode::Issues, SearchMode::Issues) => true,
            (SearchMode::CommitsAndIssues, SearchMode::CommitsAndIssues) => true,
            _ => false,
        }
    }
}

impl SearchAgent {
    pub fn new(model: ChatModel) -> SearchAgent {
        let git_client = git::Client::new();
        let github_client = github::Client::new();
        let author_mention_classifier = AuthorMentionBinaryClassifier::new(model.clone());
        let datetime_mention_classifier = DateTimeMentionClassifier::new(model.clone());
        SearchAgent {
            git_client,
            github_client,
            model,
            author_mention_classifier,
            datetime_mention_classifier,
        }
    }

    pub async fn search(
        &self,
        query: String,
        max_num_results: usize,
        mode: SearchMode,
    ) -> Result<(Vec<Commit>, Vec<Issue>), Box<dyn std::error::Error>> {
        // TODO: add concurrency
        let mut commit_results: Vec<Commit> = Vec::new();
        let mut issue_results: Vec<Issue> = Vec::new();
        if mode == SearchMode::Commits || mode == SearchMode::CommitsAndIssues {
            let all_git_commits = self.git_client.get_all_commits().unwrap();
            let store = Store::from(all_git_commits);
            let commit_retriever = BM25Retriever::new();
            commit_results = commit_retriever
                .retrieve(query.clone(), store, max_num_results)
                .unwrap();
        }
        if mode == SearchMode::Issues || mode == SearchMode::CommitsAndIssues {
            let all_github_issues = self.github_client.get_all_issues().unwrap();
            let store = Store::<Issue>::from(all_github_issues);
            let issue_retriever = BM25Retriever::new();
            issue_results = issue_retriever
                .retrieve(query.clone(), store, max_num_results)
                .unwrap();
        }
        // let all_authors = self.git_client.get_all_authors().unwrap();
        // let author_classification_result: BinaryClassificationResult<Author> = self
        //     .author_mention_classifier
        //     .classify(query.clone())
        //     .await
        //     .unwrap();
        // let datetime_classification_result: BinaryClassificationResult<(
        //     DateTime<Local>,
        //     DateTime<Local>,
        // )> = self
        //     .datetime_mention_classifier
        //     .classify(query.clone())
        //     .await
        //     .unwrap();
        Ok((commit_results, issue_results))
    }
}
