use crate::llm::{
    get_chat_completion, ChatError, ChatModel, Function, Message, Parameters, Property, Role, Tool,
    ToolCall,
};
use crate::prompts::BASE_CONTEXT_PROMPT;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;

pub enum ClassificationType {
    Binary,
    MultiClass,
}

pub trait Classifier<T> {
    fn classify(&self) -> impl Future<Output = Result<T, ChatError>> + Send;
}

pub struct MentionClassifier {
    model: ChatModel,
    api_key: String,
    query: String,
    kind: MentionKind,
}

pub enum MentionKind {
    Author,
    Since,
}

const BINARY_CLASSIFICATION_SYSTEM_PROMPT: &str = r#"# Task
You will be given a user query, an instruction, and a tool call output format to follow.

Your job is to read the relevant context, submit your answer to the instruction in the tool call format."#;

impl MentionClassifier {
    pub fn new(
        model: ChatModel,
        api_key: String,
        query: String,
        kind: MentionKind,
    ) -> MentionClassifier {
        MentionClassifier {
            model,
            api_key,
            query,
            kind,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MentionResult {
    pub classification: bool,
    pub content: Option<String>,
}

impl Classifier<MentionResult> for MentionClassifier {
    async fn classify(&self) -> Result<MentionResult, ChatError> {
        let instruction: String;
        let classifier_name: String;
        match self.kind {
            MentionKind::Author => {
                instruction = String::from(
                    "Determine if the user's query is trying to filter by the author.",
                );
                classifier_name = String::from("author_mention_classifier");
            }
            MentionKind::Since => {
                instruction =
                    String::from("Determine if the user's query is trying to filter by the date.");
                classifier_name = String::from("date_mention_classifier");
            }
        }
        let mut function_properties = HashMap::new();
        function_properties.insert(
            String::from("classification"),
            Property {
                r#type: String::from("boolean"),
                description: String::from(
                    "Whether or not the user is trying to filter by the author.",
                ),
            },
        );
        function_properties.insert(
            String::from("author_name"),
            Property {
                r#type: String::from("string"),
                description: String::from(
                    "The name of the author that the user is trying to filter by (if classification == true)",
                ),
            },
        );
        let tool = Tool {
            r#type: String::from("function"),
            function: Function {
                name: classifier_name,
                description: instruction,
                parameters: Parameters {
                    r#type: String::from("object"),
                    properties: function_properties,
                    required: vec![String::from("classification")],
                },
            },
        };
        let messages = vec![
            Message {
                role: Role::System,
                content: String::from(BASE_CONTEXT_PROMPT) + BINARY_CLASSIFICATION_SYSTEM_PROMPT,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!("# User query\n{}", self.query),
                tool_call_id: None,
            },
        ];
        let response = get_chat_completion(
            messages,
            Some(vec![tool]),
            self.model.clone(),
            &self.api_key,
        )
        .await;
        let tool_call: ToolCall = match response {
            Ok(response) => {
                let choice = &response.choices[0];
                let x = choice.message.clone();
                x.tool_calls.unwrap()[0].clone()
            }
            Err(e) => {
                return Err(e);
            }
        };
        Ok(serde_json::from_str::<MentionResult>(
            tool_call.function.arguments.as_str(),
        )?)
    }
}
