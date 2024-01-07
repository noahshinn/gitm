use crate::classifier::{BinaryClassificationResult, BinaryClassifier, LLMBinaryClassifierContext};
use crate::git::Author;
use crate::llm::{ChatError, ChatModel, Property};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct AuthorMentionBinaryClassifier {
    existing_authors: HashSet<Author>,
    raw_classifier: LLMBinaryClassifierContext,
}

impl AuthorMentionBinaryClassifier {
    pub fn new(model: ChatModel, existing_authors: HashSet<Author>) -> Self {
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
            .additional_information(format!("## Complete Author List\n{}\n\n*The author name must be an exact match to the author's name in the list above.*", 
                existing_authors
                    .iter()
                    .filter(|author| author.name.is_some())
                    .map(|author| format!("- {}", author.name.clone().unwrap()))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
            .build();
        Self {
            raw_classifier,
            existing_authors,
        }
    }
}

impl BinaryClassifier<Author> for AuthorMentionBinaryClassifier {
    async fn classify(
        &self,
        query: String,
    ) -> Result<BinaryClassificationResult<Author>, ChatError> {
        let result = self.raw_classifier.raw_classification(query).await;
        #[derive(Debug, Serialize, Deserialize)]
        struct RawResult {
            classification: bool,
            author_name: Option<String>,
        }
        match result {
            Ok(tool_call) => {
                let result =
                    serde_json::from_str::<RawResult>(tool_call.function.arguments.as_str());
                match result {
                    Ok(result) => {
                        if result.classification {
                            match result.author_name {
                                Some(author_name) => {
                                    let author = Author {
                                        name: Some(author_name.clone()),
                                        email: None,
                                        username: None,
                                    };
                                    if self.existing_authors.contains(&author) {
                                        Ok(BinaryClassificationResult {
                                            classification: true,
                                            content: Some(author),
                                        })
                                    } else {
                                        Ok(BinaryClassificationResult {
                                            classification: false,
                                            content: None,
                                        })
                                    }
                                }
                                None => {
                                    return Ok(BinaryClassificationResult {
                                        classification: false,
                                        content: None,
                                    })
                                }
                            }
                        } else {
                            Ok(BinaryClassificationResult {
                                classification: false,
                                content: None,
                            })
                        }
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
    ) -> Result<BinaryClassificationResult<(DateTime<Utc>, DateTime<Utc>)>, ChatError> {
        let result = self.raw_classifier.raw_classification(query).await;
        #[derive(Debug, Serialize, Deserialize)]
        struct RawResult {
            classification: bool,
            since: Option<String>,
            until: Option<String>,
        }
        match result {
            Ok(tool_call) => {
                let result =
                    serde_json::from_str::<RawResult>(tool_call.function.arguments.as_str());
                match result {
                    Ok(_) => {
                        // todo!("Convert RawResult to BinaryClassificationResult<(DateTime<Local>, DateTime<Local>)>")
                        Ok(BinaryClassificationResult {
                            classification: false,
                            content: None,
                        })
                    }
                    Err(e) => Err(ChatError::from(e)),
                }
            }
            Err(e) => Err(e),
        }
    }
}
