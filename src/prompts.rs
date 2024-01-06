pub const BASE_CONTEXT_PROMPT: &str = r#"# General Context
You are a git and GitHub search assistant. The specific task to complete is explained below. You must follow the instructions.
"#;

pub const BINARY_CLASSIFICATION_SYSTEM_PROMPT: &str = r#"# Task
You will be given a user query, an instruction, and a tool call output format to follow.

Your job is to read the relevant context, submit your answer to the instruction in the tool call format."#;
