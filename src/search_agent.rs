use crate::classifiers::{MentionClassifier, MentionKind};
use crate::llm::ChatModel;
use crate::store::Store;

pub struct SearchAgent {
    store: Store<String>,
    retriever: String,
    model: ChatModel,
    author_mention_classifier: MentionClassifier,
    datetime_mention_classifier: MentionClassifier,
}

impl SearchAgent {
    pub fn new(store: Store<String>, retriever: String, model: ChatModel) -> Self {
        let author_mention_classifier = MentionClassifier::new(model.clone(), MentionKind::Author);
        let datetime_mention_classifier =
            MentionClassifier::new(model.clone(), MentionKind::DateTime);
        Self {
            store,
            retriever,
            model,
            author_mention_classifier,
            datetime_mention_classifier,
        }
    }

    pub fn search(&self, query: String) -> Vec<String> {
        todo!("Implement search")
    }
}
