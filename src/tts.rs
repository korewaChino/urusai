use std::{collections::HashMap};
use bytes::{Bytes};
use anyhow::{ Result};
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

#[derive(Debug)]
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
        // I'd use Serde to deserialize this, but thing is that the Bytes object below is not serializable
        name: String,
        error: i32,
        speaker: String,
        cached: i32,
        tasktype: String,
        url: String,
        mp3: String,
    },
    OmameSAPI { // Uses Omame's SAPI Online API at https://sapi.omame.xyz/api/
        name: String,
        // It just returns raw data, so we don't need to parse it.
        data: Bytes,
    }
}


impl TTS {
    pub async fn request(lang: &str, text: &str, msg: Option<&Message>) -> Result<Self, String> {
        let client = Client::new();
        // Name format: server_id-channel_id-user_id
        let name = if let Some(msg) = msg {
            let mut path = std::env::current_dir().unwrap();
            path.push("tts");
            let channelid = msg.channel_id.0;
            let serverid = msg.guild_id.unwrap().0;
            path.push(&serverid.to_string());
            path.push(&channelid.to_string());
            std::fs::create_dir_all(path).unwrap();
            format!("{}/{}/{}.mp3", msg.guild_id.unwrap().0, msg.channel_id.0, msg.author.id.0)
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
            let tiktok = TTS::TikTok {
                name,
                data: Some(res["data"].clone()),
                extra: Some(res["extra"].clone()),
                message: res["message"].as_str().unwrap().to_string(),
                status_code: res["status_code"].as_i64().unwrap() as i32,
                status_msg: res["status_msg"].as_str().unwrap().to_string(),
            };
            Ok(tiktok)
        } else if lang.starts_with("ttsmp3-") {
            // ttsmp3
            let lang = lang.split("-").nth(1).unwrap();
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
            // Check if there is an error
            if res["Error"].as_i64().unwrap() != 0 {
                return Err(res["Error"].as_str().unwrap().to_string());
            }
            let ttsmp3 = TTS::TTSMP3 {
                name,
                error: res["Error"].as_i64().unwrap_or_default() as i32,
                speaker: res["Speaker"].as_str().unwrap().to_string(),
                cached: res["Cached"].as_i64().unwrap() as i32,
                tasktype: res["tasktype"].as_str().unwrap().to_string(),
                url: res["URL"].as_str().unwrap_or_default().to_string(),
                mp3: res["MP3"].as_str().unwrap_or_default().to_string(),
            };
            Ok(ttsmp3)
        } else if lang.starts_with("sapi-") {
            // sapi
            let lang = lang.split("-").nth(1).unwrap();
            let param = [("msg", text), ("voice", lang)];

            let res = client
                .post("https://sapi.omame.xyz/api/")
                .query(&param)
                .send()
                .await
                .unwrap();

            // This directly returns the data
            let data = res.bytes().await.unwrap();
            Ok(
                TTS::OmameSAPI {
                    name,
                    data
                }
            )

        } else {
            Err("Unknown Voice".to_string())
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
                Ok(name)
            }

            TTS::OmameSAPI { name, data, .. } => {
                // Write the data to a file
                //let name = name.replace(".mp3", ".wav");
                let name = format!("tts/{}", name);
                let mut file = tokio::fs::File::create(name.clone()).await.expect("Failed to create file");
                let mut content =  Cursor::new(data);
                tokio::io::copy(&mut content, &mut file).await.expect("Failed to copy file");
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
        let lang = "ttsmp3-Justin";
        let tts = TTS::request(&lang, &text, None).await.unwrap();
        tts.download().await.unwrap();
    }
}