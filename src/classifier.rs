use crate::llm::{
    ChatError, ChatModel, Function, Message, Parameters, Property, Role, Tool, ToolCall,
};
use crate::prompts::{BASE_CONTEXT_PROMPT, BINARY_CLASSIFICATION_SYSTEM_PROMPT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;

pub trait BinaryClassifier<T> {
    fn classify(
        &self,
        query: String,
    ) -> impl Future<Output = Result<BinaryClassificationResult<T>, ChatError>> + Send;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryClassificationResult<T> {
    pub classification: bool,
    pub content: Option<T>,
}

pub struct LLMBinaryClassifierContext {
    model: ChatModel,
    system_prompt: String,
    instruction: String,
    result_properties: Vec<(String, Property)>,
}

impl LLMBinaryClassifierContext {
    pub fn new(
        model: ChatModel,
        instruction: String,
        additional_information: String,
        result_properties: Vec<(String, Property)>,
    ) -> Self {
        Self {
            model,
            system_prompt: format!(
                "{}\n\n{}{}",
                BASE_CONTEXT_PROMPT,
                BINARY_CLASSIFICATION_SYSTEM_PROMPT,
                if additional_information.len() > 0 {
                    format!("\n\n# Additional Information\n{}", additional_information)
                } else {
                    String::from("")
                }
            ),
            instruction,
            result_properties,
        }
    }

    pub fn builder(model: ChatModel, instruction: String) -> LLMBinaryClassifierContextBuilder {
        LLMBinaryClassifierContextBuilder::new(model, instruction)
    }

    pub async fn raw_classification(&self, query: String) -> Result<ToolCall, ChatError> {
        let mut function_properties = HashMap::new();
        function_properties.insert(
            String::from("classification"),
            Property {
                r#type: String::from("boolean"),
                description: String::from(
                    "The binary classification derived from the context and instruction",
                ),
            },
        );
        for (name, property) in self.result_properties.iter() {
            function_properties.insert(
                name.clone(),
                Property {
                    r#type: property.r#type.clone(),
                    description: format!("{} (if classification == true)", property.description),
                },
            );
        }
        let tool = Tool {
            r#type: String::from("function"),
            function: Function {
                name: String::from("binary_classification"),
                description: self.instruction.clone(),
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
                content: self.system_prompt.clone(),
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!(
                    "# User query\n{}\n\n# Instruction\n{}",
                    query, self.instruction
                ),
                tool_call_id: None,
            },
        ];
        let response = self.model.chat(messages, Some(vec![tool]), 0.0).await;
        match response {
            Ok(response) => {
                let choice = &response.choices[0];
                let x = choice.message.clone();
                let tool_call = x.tool_calls.unwrap()[0].clone();
                Ok(tool_call)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

pub struct LLMBinaryClassifierContextBuilder {
    model: ChatModel,
    instruction: String,
    additional_information: String,
    result_properties: Vec<(String, Property)>,
}

impl LLMBinaryClassifierContextBuilder {
    fn new(model: ChatModel, instruction: String) -> Self {
        Self {
            model,
            instruction,
            result_properties: vec![],
            additional_information: String::from(""),
        }
    }

    pub fn result_property(mut self, result_property: (String, Property)) -> Self {
        self.result_properties.push(result_property);
        self
    }

    pub fn additional_information(mut self, additional_information: String) -> Self {
        self.additional_information = additional_information;
        self
    }

    pub fn build(self) -> LLMBinaryClassifierContext {
        LLMBinaryClassifierContext::new(
            self.model,
            self.instruction,
            self.additional_information,
            self.result_properties,
        )
    }
}
