use crate::classifier::{BinaryClassificationResult, BinaryClassifier};
use crate::git::{Author, Client, Commit};
use crate::llm::ChatModel;
use crate::mention_classifiers::{AuthorMentionBinaryClassifier, DateTimeMentionClassifier};
use crate::retrievers::Retriever;
use crate::store::Store;
use chrono::{DateTime, Local};

pub struct SearchAgent<'a> {
    git_client: Client,
    retriever: &'a dyn Retriever<String, Commit>,
    model: ChatModel,
    author_mention_classifier: AuthorMentionBinaryClassifier,
    datetime_mention_classifier: DateTimeMentionClassifier,
}

impl SearchAgent<'_> {
    pub fn new(
        git_client: Client,
        retriever: &dyn Retriever<String, Commit>,
        model: ChatModel,
    ) -> SearchAgent {
        let author_mention_classifier = AuthorMentionBinaryClassifier::new(model.clone());
        let datetime_mention_classifier = DateTimeMentionClassifier::new(model.clone());
        SearchAgent {
            git_client,
            retriever,
            model,
            author_mention_classifier,
            datetime_mention_classifier,
        }
    }

    pub async fn search(&self, query: String, max_num_results: usize) -> Vec<Commit> {
        // TODO: add concurrency
        let all_git_commits = self.git_client.get_all_commits().unwrap();
        // let all_authors = self.git_client.get_all_authors().unwrap();
        let store = Store::<Commit>::from(all_git_commits);
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
        let results = self.retriever.retrieve(query, store, max_num_results);
        match results {
            Ok(results) => results,
            Err(e) => {
                println!("Error: {}", e);
                Vec::new()
            }
        }
    }
}
