#![allow(dead_code)]

use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;

#[derive(Debug, Deserialize)]
struct ChatChunkDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkChoice {
    delta: ChatChunkDelta,
    index: usize,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    id: String,
    object: String,
    created: usize,
    model: String,
    choices: Vec<ChatChunkChoice>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let url = "https://api.openai.com/v1/chat/completions";
    let api_key = std::env::var("OPENAI_API_KEY")?;

    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{
            "role": "user",
            "content": "Please list 10 things I might want to bring to a picnic."
        }],
        "stream": true
    });

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(body.to_string())
        .header("Content-Type", "application/json")
        .bearer_auth(api_key)
        .send()
        .await?;
    println!("status = {}", res.status());

    println!("ChatGPT Says:");
    println!();
    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        let item = item?;
        let s = match std::str::from_utf8(&item) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        for p in s.split("\n\n") {
            match p.strip_prefix("data: ") {
                Some(p) => {
                    // Check if the stream is done...
                    if p == "[DONE]" {
                        break;
                    }

                    // Parse the json data...
                    let d = serde_json::from_str::<ChatCompletionChunk>(p)
                        .expect(format!("Couldn't parse: {}", p).as_str());

                    // Is there data?
                    let c = d.choices.get(0).expect("No choice returned");
                    if let Some(content) = &c.delta.content {
                        print!("{}", content);
                    }

                    // Flush stdout as it goes...
                    if let Err(error) = std::io::stdout().flush() {
                        panic!("{}", error);
                    }
                }
                None => {}
            }
        }
    }
    println!("");
    println!("[Done.]");

    Ok(())
}
