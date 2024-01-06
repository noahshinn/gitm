use crate::classifier::{BinaryClassificationResult, BinaryClassifier, LLMBinaryClassifierContext};
use crate::llm::{ChatError, ChatModel, Property};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub struct AuthorMentionBinaryClassifier {
    raw_classifier: LLMBinaryClassifierContext,
}

impl AuthorMentionBinaryClassifier {
    pub fn new(model: ChatModel) -> Self {
        let instruction =
            String::from("Determine if the user's query is trying to filter by the author.");
        let result_property = (
            String::from("author_name"),
            Property {
                r#type: String::from("string"),
                description: String::from(
                    "The name of the author that the user is trying to filter by (if classification == true)",
                ),
            },
        );
        let raw_classifier = LLMBinaryClassifierContext::builder(model, instruction)
            .result_property(result_property)
            .additional_information(String::from(
                "The author name must be an exact match to the author's name in the git log.",
            ))
            .build();
        Self { raw_classifier }
    }
}

impl<Author> BinaryClassifier<Author> for AuthorMentionBinaryClassifier {
    async fn classify(
        &self,
        query: String,
    ) -> Result<BinaryClassificationResult<Author>, ChatError> {
        let result = self.raw_classifier.raw_classification(query).await;
        #[derive(Debug, Serialize, Deserialize)]
        struct RawResult {
            classification: bool,
            author_name: String,
        }
        match result {
            Ok(tool_call) => {
                let result =
                    serde_json::from_str::<RawResult>(tool_call.function.arguments.as_str());
                match result {
                    Ok(_) => {
                        todo!("Convert RawResult to BinaryClassificationResult<Author>")
                    }
                    Err(e) => Err(ChatError::from(e)),
                }
            }
            Err(e) => Err(e),
        }
    }
}

pub struct DateTimeMentionClassifier {
    raw_classifier: LLMBinaryClassifierContext,
}

impl DateTimeMentionClassifier {
    pub fn new(model: ChatModel) -> Self {
        let instruction =
            String::from("Determine if the user's query is trying to filter by the date.");
        let before_date_property = (
            String::from("since"),
            Property {
                r#type: String::from("string"),
                description: String::from(
                    "The YYYY-MM-DD since date that the user is trying to filter by",
                ),
            },
        );
        let after_date_property = (
            String::from("until"),
            Property {
                r#type: String::from("string"),
                description: String::from(
                    "The YYYY-MM-DD until date that the user is trying to filter by",
                ),
            },
        );
        let current_datetime = chrono::Local::now();
        let raw_classifier = LLMBinaryClassifierContext::builder(model, instruction)
            .result_property(before_date_property)
            .result_property(after_date_property)
            .additional_information(format!(
                "The date must be in the format YYYY-MM-DD.\n\nThe current datetime is: {}",
                current_datetime.format("%Y-%m-%d").to_string()
            ))
            .build();
        Self { raw_classifier }
    }

    pub async fn classify(
        &self,
        query: String,
    ) -> Result<BinaryClassificationResult<(DateTime<Local>, DateTime<Local>)>, ChatError> {
        let result = self.raw_classifier.raw_classification(query).await;
        #[derive(Debug, Serialize, Deserialize)]
        struct RawResult {
            classification: bool,
            since: String,
            until: String,
        }
        match result {
            Ok(tool_call) => {
                let result =
                    serde_json::from_str::<RawResult>(tool_call.function.arguments.as_str());
                match result {
                    Ok(_) => {
                        todo!("Convert RawResult to BinaryClassificationResult<(DateTime<Local>, DateTime<Local>)>")
                    }
                    Err(e) => Err(ChatError::from(e)),
                }
            }
            Err(e) => Err(e),
        }
    }
}
