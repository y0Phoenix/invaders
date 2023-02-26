use std::{error::Error, env};

use reqwest::Client;

use crate::system::SystemPlayer;

pub struct ReqClient {
    client: Client,
    api_key: String,
    url: String
}

impl ReqClient {
    pub fn new() -> Self {
        let api_key = match env::var("API_KEY") {
            Ok(key) => key,
            Err(_) => String::new()
        };
        // let url = "https://invaders-server.herokuapp.com/".to_string();
        let url = "http://127.0.1.0:3000".to_string();
        Self { client: Client::new(), api_key, url}
    }
    pub async fn get_scores(&self) -> Result<Vec<SystemPlayer>, Box<dyn Error>> {
        let response = match self.client.get(self.url.as_str())
        .header("x-api-key", self.api_key.as_str())
        .send()
        .await {
            Ok(res) => res,
            Err(_) => return Err("Couldn't Connect To Server".into()) 
        };
    
        let body = response.text().await.unwrap();

        let high_scores = match serde_json::from_str::<Vec<SystemPlayer>>(&body) {
            Ok(scores) => scores,
            Err(_) => return Err(body.into())
        };
        Ok(high_scores)
    }
    pub async fn update_scores(&self, score: SystemPlayer) -> Result<String, Box<dyn Error>> {
        let body = match serde_json::to_string(&score) {
            Ok(data) => data,
            Err(_) => return Err("There Was A Problem Parsing Score Data".into())
        };
        let response = match self.client.post(self.url.as_str())
            .header("Content-Type", "application/json")
            .header("x-api-key", self.api_key.as_str())
            .body(body)
            .send()
            .await {
            Ok(res) => res,
            Err(_) => return Err("Couldn't Connect To Server".into())  
        };
        let body = response.text().await.unwrap();

        Ok(body)
    }
}