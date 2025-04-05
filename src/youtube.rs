use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tokio::time::{sleep, Duration};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub author: String,
    pub text: String,
    pub timestamp: String,
}

pub struct ChatMonitor {
    client: Client,
    video_id: String,
    api_key: String,
    next_page_token: Option<String>,
    poll_interval_ms: u64,
}

impl ChatMonitor {
    pub fn new(video_id: &str, api_key: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(AppError::YouTube("API key is required".to_string()).into());
        }

        Ok(Self {
            client: Client::new(),
            video_id: video_id.to_string(),
            api_key: api_key.to_string(),
            next_page_token: None,
            poll_interval_ms: 3000,
        })
    }

    pub fn set_poll_interval(&mut self, ms: u64) {
        self.poll_interval_ms = ms;
    }

    pub async fn next_message(&mut self) -> Result<Option<ChatMessage>> {
        if self.next_page_token.is_none() {
            // Initial fetch to get live chat ID
            self.initialize_chat().await?
        }

        loop {
            let messages = self.fetch_messages().await?;

            if !messages.is_empty() {
                // Return first message and keep the rest for the next call
                return Ok(Some(messages[0].clone()));
            }

            // Wait before polling again
            sleep(Duration::from_millis(self.poll_interval_ms)).await;
        }
    }

    async fn initialize_chat(&mut self) -> Result<()> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=liveStreamingDetails&id={}&key={}",
            self.video_id, self.api_key
        );

        let response = self.client.get(&url).send().await?
            .json::<serde_json::Value>().await?;

        let items = response["items"].as_array()
            .ok_or_else(|| AppError::YouTube("Invalid API response".to_string()))?;

        if items.is_empty() {
            return Err(AppError::YouTube("Video not found or not a live stream".to_string()).into());
        }

        // Store the live chat ID
        let chat_id = items[0]["liveStreamingDetails"]["activeLiveChatId"].as_str()
            .ok_or_else(|| AppError::YouTube("Live chat not available".to_string()))?
            .to_string();

        // Store the chat ID in the video_id field temporarily
        self.video_id = chat_id;
        self.next_page_token = Some(String::new());
        Ok(())
    }

    async fn fetch_messages(&mut self) -> Result<Vec<ChatMessage>> {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/liveChat/messages?part=snippet,authorDetails&liveChatId={}&key={}",
            self.video_id, self.api_key
        );

        if let Some(token) = &self.next_page_token {
            if !token.is_empty() {
                url.push_str(&format!("&pageToken={}", token));
            }
        }

        let response = self.client.get(&url).send().await?
            .json::<serde_json::Value>().await?;

        // Update next page token
        self.next_page_token = response["nextPageToken"].as_str().map(String::from);

        let items = match response["items"].as_array() {
            Some(items) => items,
            None => return Ok(vec![]),
        };

        let mut messages = Vec::new();
        for item in items {
            if let (Some(id), Some(author), Some(text), Some(timestamp)) = (
                item["id"].as_str(),
                item["authorDetails"]["displayName"].as_str(),
                item["snippet"]["displayMessage"].as_str(),
                item["snippet"]["publishedAt"].as_str(),
            ) {
                messages.push(ChatMessage {
                    id: id.to_string(),
                    author: author.to_string(),
                    text: text.to_string(),
                    timestamp: timestamp.to_string(),
                });
            }
        }

        Ok(messages)
    }
}
