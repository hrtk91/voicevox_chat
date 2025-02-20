use std::sync::Arc;

use colored::Colorize;
use serde::Serialize;

pub struct ChatCompletion {
    api_key: String,
    client: Arc<reqwest::Client>,
    model: String,
    log_size: u32,
    system_messages: Vec<Message>,
    chat_messages: Vec<Message>,
}

#[derive(Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl ChatCompletion {
    pub fn new(api_key: String, client: Arc<reqwest::Client>) -> Self {
        Self {
            api_key,
            client,
            model: "gpt-4o-mini".to_string(),
            log_size: 30,
            system_messages: vec![],
            chat_messages: vec![],
        }
    }

    pub fn push_system_message(&mut self, prompt: &str) {
        let system_message = Message {
            role: "system".into(),
            content: prompt.into(),
        };

        self.system_messages.push(system_message);
    }

    fn push_chat_message(&mut self, role: &str, input: &str) {
        if self.chat_messages.len() > self.log_size as usize {
            self.chat_messages.remove(0);
        }

        self.chat_messages.push(Message {
            role: role.into(),
            content: input.into(),
        });
    }

    pub fn push_user_message(&mut self, input: &str) {
        self.push_chat_message("user", input);
    }

    pub fn push_assistant_message(&mut self, input: &str) {
        self.push_chat_message("assistant", input);
    }

    pub fn messages(&self) -> Vec<Message> {
        self.system_messages
            .iter()
            .cloned()
            .chain(self.chat_messages.clone())
            .collect()
    }

    pub async fn completion(&self) -> Result<String, reqwest::Error> {
        let body = serde_json::json!({
          "model": "gpt-4o-mini",
          "messages": self.messages()
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await;

        let resp = match resp {
            // レスポンスのステータスコードを確認
            Ok(response) => match response.error_for_status() {
                Ok(valid_response) => valid_response,
                Err(e) => {
                    eprintln!("{}", format!("Error in response status: {}", e).red());
                    return Err(e);
                }
            },
            Err(e) => {
                eprintln!("{}", format!("Error sending request: {}", e).red());
                return Err(e);
            }
        };

        let resp_json: serde_json::Value = resp.json().await?;
        let choices = resp_json["choices"]
            .as_array()
            .expect("choices is not an array");
        let text = choices[0]["message"]["content"]
            .as_str()
            .expect("content is not a string");

        Ok(text.to_string())
    }
}

impl ChatCompletion {
    pub fn api_key(&mut self, api_key: &str) -> &mut Self {
        self.api_key = api_key.to_string();
        self
    }

    pub fn model(&mut self, model_name: &str) -> &mut Self {
        self.model = model_name.to_string();
        self
    }

    pub fn client(&mut self, client: Arc<reqwest::Client>) -> &mut Self {
        self.client = client;
        self
    }

    pub fn log_size(&mut self, size: u32) -> &mut Self {
        self.log_size = size;
        self
    }
}
