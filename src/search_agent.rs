use crate::bm25::{BM25Ranker, BM25Retriever};
use crate::classifier::{BinaryClassificationResult, BinaryClassifier};
use crate::git;
use crate::git::{Author, Commit};
use crate::github;
use crate::github::Issue;
use crate::llm::ChatModel;
use crate::mention_classifiers::{AuthorMentionBinaryClassifier, DateTimeMentionClassifier};
use crate::retrievers::Retriever;
use crate::splitters::PuncSplitter;
use crate::store::Store;
use chrono::{DateTime, Local};
use std::collections::HashSet;

pub struct SearchAgent {
    git_client: git::Client,
    github_client: github::Client,
    model: ChatModel,
    author_mention_classifier: AuthorMentionBinaryClassifier,
    datetime_mention_classifier: DateTimeMentionClassifier,
}

pub struct SearchConfig {
    query: String,
    max_num_results: usize,
    include_commits: bool,
    include_issues: bool,
    include_code_patches: bool,
}

pub struct SearchConfigBuilder {
    query: String,
    max_num_results: usize,
    include_commits: bool,
    include_issues: bool,
    include_code_patches: bool,
}

impl SearchConfigBuilder {
    pub fn new(query: String) -> SearchConfigBuilder {
        SearchConfigBuilder {
            query,
            max_num_results: 10,
            include_commits: true,
            include_issues: false,
            include_code_patches: false,
        }
    }

    pub fn max_num_results(mut self, max_num_results: usize) -> SearchConfigBuilder {
        self.max_num_results = max_num_results;
        self
    }

    pub fn include_commits(mut self, include_commits: bool) -> SearchConfigBuilder {
        self.include_commits = include_commits;
        self
    }

    pub fn include_issues(mut self, include_issues: bool) -> SearchConfigBuilder {
        self.include_issues = include_issues;
        self
    }

    pub fn include_code_patches(mut self, include_code_patches: bool) -> SearchConfigBuilder {
        self.include_code_patches = include_code_patches;
        self
    }

    pub fn build(self) -> SearchConfig {
        SearchConfig {
            query: self.query,
            max_num_results: self.max_num_results,
            include_commits: self.include_commits,
            include_issues: self.include_issues,
            include_code_patches: self.include_code_patches,
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
        search_config: SearchConfig,
    ) -> Result<(Vec<Commit>, Vec<Issue>), Box<dyn std::error::Error>> {
        // TODO: add concurrency
        let mut commit_results: Vec<Commit> = Vec::new();
        let mut issue_results: Vec<Issue> = Vec::new();
        if search_config.include_commits {
            let all_git_commits = self.git_client.get_all_commits().unwrap();
            // TODO: remove this heavy clone
            let store = Store::from(all_git_commits.clone());
            let commit_retriever = BM25Retriever::new();
            commit_results = commit_retriever
                .retrieve(
                    search_config.query.clone(),
                    store,
                    search_config.max_num_results,
                )
                .unwrap();
            if search_config.include_code_patches {
                let code_commits: Vec<Commit> = all_git_commits
                    .iter()
                    .map(|commit| Commit {
                        display_mode: git::CommitDisplayMode::PatchSetAdd,
                        ..commit.to_owned()
                    })
                    .collect();
                let store = Store::<Commit>::from(code_commits);
                let code_ranker = BM25Ranker::builder().splitter(&PuncSplitter).build();
                let commit_retriever = BM25Retriever::builder().ranker(code_ranker).build();
                commit_results.append(
                    &mut commit_retriever
                        .retrieve(
                            search_config.query.clone(),
                            store,
                            search_config.max_num_results,
                        )
                        .unwrap(),
                );
            }
            // Note: this is a hack to dedupe the results as the size of the results will be small
            let mut deduped: Vec<Commit> = Vec::new();
            let mut seen: HashSet<String> = HashSet::new();
            for commit in commit_results {
                if !seen.contains(&commit.sha) {
                    seen.insert(commit.sha.clone());
                    deduped.push(commit);
                }
            }
            commit_results = deduped;
        }
        if search_config.include_issues {
            let all_github_issues = self.github_client.get_all_issues().unwrap();
            let store = Store::<Issue>::from(all_github_issues);
            let issue_retriever = BM25Retriever::new();
            issue_results = issue_retriever
                .retrieve(
                    search_config.query.clone(),
                    store,
                    search_config.max_num_results,
                )
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
