use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ChatModel {
    Gpt35Turbo,
    Gpt4,
}

impl fmt::Display for ChatModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatModel::Gpt35Turbo => write!(f, "gpt-3.5-turbo"),
            ChatModel::Gpt4 => write!(f, "gpt-4-0613"),
        }
    }
}

impl Serialize for ChatModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ChatModel {
    fn deserialize<D>(deserializer: D) -> Result<ChatModel, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "gpt-3.5-turbo" => Ok(ChatModel::Gpt35Turbo),
            "gpt-4-0613" => Ok(ChatModel::Gpt4),
            _ => Err(serde::de::Error::custom(format!(
                "unknown chat model: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequestBody {
    pub model: ChatModel,
    pub messages: Vec<Message>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Role::System => serializer.serialize_str("system"),
            Role::User => serializer.serialize_str("user"),
            Role::Assistant => serializer.serialize_str("assistant"),
            Role::Tool => serializer.serialize_str("tool"),
        }
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Role, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "system" => Ok(Role::System),
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "tool" => Ok(Role::Tool),
            _ => Err(serde::de::Error::custom(format!("unknown role: {}", s))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub r#type: String,
    pub function: Function,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub r#type: String,
    pub properties: HashMap<String, Property>,
    pub required: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    pub r#type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";
const OPENAI_API_CHAT_ENDPOINT: &str = "/chat/completions";

#[derive(Debug)]
pub struct ChatError {
    pub message: String,
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChatError: {}", self.message)
    }
}

impl From<serde_json::Error> for ChatError {
    fn from(error: serde_json::Error) -> Self {
        ChatError {
            message: format!("{}", error),
        }
    }
}

pub async fn get_chat_completion(
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
    model: ChatModel,
    api_key: &str,
) -> Result<ChatResponse, ChatError> {
    let url = format!("{}{}", OPENAI_API_BASE, OPENAI_API_CHAT_ENDPOINT);
    let client = reqwest::Client::new();
    let body = ChatRequestBody {
        model,
        messages,
        temperature: 0.9,
        tools,
    };
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await;
    match response {
        Ok(response) => {
            if response.status() != reqwest::StatusCode::OK {
                return Err(ChatError {
                    message: format!("{}: {}", response.status(), response.text().await.unwrap()),
                });
            }
            let chat_response = response.json::<ChatResponse>().await.unwrap();
            return Ok(chat_response);
        }
        Err(e) => {
            return Err(ChatError {
                message: format!("{}", e),
            });
        }
    };
}
