use std::collections::HashMap;

use anyhow::{Ok, Result};
use reqwest::{Client, Response};
use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use serenity::model::channel::Message;
use tokio::{fs::File, io::AsyncWriteExt};
use std::io::Cursor;
static TIKTOK_API_URL: &str = "https://api16-normal-useast5.us.tiktokv.com/media/api/text/speech/invoke/";
static TTSMP3_API_URL: &str = "https://ttsmp3.com/makemp3_new.php";
/* msg: your text here
lang: voice
source: ttsmp3
then get the URL from the json output
easy */

#[derive(Debug, Serialize, Deserialize)]
pub enum TTS {
    TikTok {
        name: String,
        data: Option<Value>,
        extra: Option<Value>,
        message: String,
        status_code: i32,
        status_msg: String,
    },
    TTSMP3 {
        name: String,
        #[serde(rename = "Error")]
        error: i32,
        #[serde(rename = "Speaker")]
        speaker: String,
        #[serde(rename = "Cached")]
        cached: i32,
        tasktype: String,
        #[serde(rename = "URL")]
        url: String,
        #[serde(rename = "MP3")]
        mp3: String,
    }
}


impl TTS {
    pub async fn request(lang: &str, text: &str, msg: Option<&Message>) -> Self {
        let client = Client::new();
        // Name format: server_id-channel_id-user_id
        let name = if let Some(msg) = msg {
            format!("{}-{}-{}.mp3", msg.guild_id.unwrap().0, msg.channel_id.0, msg.author.id.0)
        } else {
            "test.mp3".to_string()
        };
        if lang.starts_with("tiktok-") {
            let lang = lang.split("-").nth(1).unwrap();
            let param = [("text_speaker", lang), ("req_text", text)];
            let res: Value = client
                .post(TIKTOK_API_URL)
                .query(&param)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            // return a TTS::TikTok
            TTS::TikTok {
                name,
                data: Some(res["data"].clone()),
                extra: Some(res["extra"].clone()),
                message: res["message"].as_str().unwrap().to_string(),
                status_code: res["status_code"].as_i64().unwrap() as i32,
                status_msg: res["status_msg"].as_str().unwrap().to_string(),
            }
        } else {
            // ttsmp3

            // urlencoded form data
            let mut params = HashMap::new();
            params.insert("msg", text);
            params.insert("lang", lang);
            params.insert("source", "ttsmp3");
            let res: Value = client
                .post(TTSMP3_API_URL)
                .form(&params)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            TTS::TTSMP3 {
                name,
                error: res["Error"].as_i64().unwrap_or_default() as i32,
                speaker: res["Speaker"].as_str().unwrap().to_string(),
                cached: res["Cached"].as_i64().unwrap() as i32,
                tasktype: res["tasktype"].as_str().unwrap().to_string(),
                url: res["URL"].as_str().unwrap_or_default().to_string(),
                mp3: res["MP3"].as_str().unwrap_or_default().to_string(),
            }
        }
    }

    pub async fn download(self) -> Result<String> {
        // Match the type
        match self {
            TTS::TikTok { name, data, .. } => {
                // So the data is a base64 encoded string, which we need to decode to an mp3 file

                let data_json = data.unwrap();
                // First, we need to get the base64 encoded string
                let data_base64 = data_json["v_str"].as_str().unwrap();

                // Then, we need to decode the base64 string
                let data_raw = base64::decode(data_base64).unwrap();

                let name = format!("tts/{}", name);
                // Write the data to a file
                let mut file = File::create(name.clone()).await.unwrap();
                file.write_all(&data_raw).await.unwrap();
                Ok(name)
            }

            TTS::TTSMP3 { name, url, .. } => {
                // Download the file from the URL
                let name = format!("tts/{}", name);
                let response = reqwest::get(url).await.unwrap();
                let mut file = tokio::fs::File::create(name.clone()).await.expect("Failed to create file");
                let mut content =  Cursor::new(response.bytes().await.unwrap());
                tokio::io::copy(&mut content, &mut file).await.expect("Failed to copy file");
                // Get the file path
                Ok(name)
            }

        }

    }
}



#[cfg(test)]
mod test_super {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test() {
        let client = Client::new();
        let params = [("text_speaker", "en_us_002"), ("req_text", "Hello, world!")];
        let res = client
            .post(TIKTOK_API_URL)
            .query(&params)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        println!("{:#?}", res);
    }
    #[tokio::test]
    async fn test_tiktok() {
        let text = "Hello, world!";
        let lang = "Justin";
        let tts = TTS::request(&lang, &text, None).await;
        tts.download().await;
    }
}