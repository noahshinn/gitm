use crate::bm25::{BM25Ranker, BM25Retriever};
use crate::classifier::{BinaryClassificationResult, BinaryClassifier};
use crate::git;
use crate::git::{Author, Commit, FilterConfig};
use crate::github;
use crate::github::Issue;
use crate::llm::ChatModel;
use crate::mention_classifiers::{AuthorMentionBinaryClassifier, DateTimeMentionClassifier};
use crate::retrievers::Retriever;
use crate::splitters::PuncSplitter;
use crate::store::Store;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

pub struct SearchAgent {
    git_client: git::Client,
    github_client: github::Client,
    author_mention_classifier: AuthorMentionBinaryClassifier,
    datetime_mention_classifier: DateTimeMentionClassifier,
}

pub struct SearchConfig {
    query: String,
    max_num_results: usize,
    include_commits: bool,
    include_issues: bool,
    include_code_patches: bool,
    disable_classifications: bool,
    search_all: bool,
}

pub struct SearchConfigBuilder {
    query: String,
    max_num_results: usize,
    include_commits: bool,
    include_issues: bool,
    include_code_patches: bool,
    disable_classifications: bool,
    search_all: bool,
}

impl SearchConfigBuilder {
    pub fn new(query: String) -> SearchConfigBuilder {
        SearchConfigBuilder {
            query,
            max_num_results: 10,
            include_commits: true,
            include_issues: false,
            include_code_patches: false,
            disable_classifications: false,
            search_all: false,
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

    pub fn disable_classifications(mut self, disable_classifications: bool) -> SearchConfigBuilder {
        self.disable_classifications = disable_classifications;
        self
    }

    pub fn search_all(mut self, search_all: bool) -> SearchConfigBuilder {
        self.search_all = search_all;
        self
    }

    pub fn build(self) -> SearchConfig {
        SearchConfig {
            query: self.query,
            max_num_results: self.max_num_results,
            include_commits: self.include_commits,
            include_issues: self.include_issues,
            include_code_patches: self.include_code_patches,
            disable_classifications: self.disable_classifications,
            search_all: self.search_all,
        }
    }
}

impl SearchAgent {
    pub fn new(model: ChatModel) -> SearchAgent {
        let git_client = git::Client::new();
        let github_client = github::Client::new();
        let all_authors = git_client.get_all_authors().unwrap().into_iter().collect();
        let author_mention_classifier =
            AuthorMentionBinaryClassifier::new(model.clone(), all_authors);
        let datetime_mention_classifier = DateTimeMentionClassifier::new(model.clone());
        SearchAgent {
            git_client,
            github_client,
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
            let mut filter_config: Option<FilterConfig> = None;
            if !search_config.disable_classifications {
                let author_classification_result: BinaryClassificationResult<Author> = self
                    .author_mention_classifier
                    .classify(search_config.query.clone())
                    .await
                    .unwrap();
                let datetime_classification_result: BinaryClassificationResult<(
                    Option<DateTime<Utc>>,
                    Option<DateTime<Utc>>,
                )> = self
                    .datetime_mention_classifier
                    .classify(search_config.query.clone())
                    .await
                    .unwrap();
                let mut filter = FilterConfig {
                    author: None,
                    date_range: None,
                    git_log_get_all: Some(search_config.search_all),
                };
                if author_classification_result.classification {
                    if let Some(author) = author_classification_result.content {
                        filter.author = Some(author);
                    }
                }
                if datetime_classification_result.classification {
                    if let Some((start_date, end_date)) = datetime_classification_result.content {
                        filter.date_range = Some((start_date, end_date));
                    }
                }
                filter_config = Some(filter);
            }
            let all_git_commits = self.git_client.get_all_commits(filter_config).unwrap();
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
        Ok((commit_results, issue_results))
    }
}
