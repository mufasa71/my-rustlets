use std::io::Read;
use std::{fs::File, path::PathBuf};

use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::chat_completion::ChatCompletionRequest;

use crate::config::Config;

pub fn read_template(template_file: &PathBuf) -> std::io::Result<String> {
    let file = File::open(template_file)?;
    let mut reader = std::io::BufReader::new(file);
    let mut contents = String::new();

    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

pub async fn generate_commit(
    content: String,
    config: Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let system_message = "You are a commit message generator. I will provide you with a git diff, and I would like you to generate an appropriate commit message using the conventional commit format. Do not write any explanations or other words, just reply with the commit message.";
    let mut client = OpenAIClient::builder()
        .with_endpoint(config.openai_api_url)
        .with_api_key(config.openai_api_key)
        .build()?;

    let req = ChatCompletionRequest::new(
        config.model_name,
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(system_message.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(content),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
    );

    let result = client.chat_completion(req).await?;
    let contents = &result.choices[0].message.content;

    match contents {
        Some(content) => Ok(content.clone()),
        None => Ok(String::from("")),
    }
}
