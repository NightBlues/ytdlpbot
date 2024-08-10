// use std::collections::hash_map::HashMap;
use std::vec::Vec;
use anyhow::{Result, Context, anyhow};


fn url_get_updates(token: &String) -> String /* dyn reqwest::IntoUrl */ {
  format!("https://api.telegram.org/bot{}/getUpdates", token)
}

fn url_send_message(token: &String) -> String /* dyn reqwest::IntoUrl */ {
  format!("https://api.telegram.org/bot{}/sendMessage", token)
}

fn url_delete_message(token: &String) -> String /* dyn reqwest::IntoUrl */ {
  format!("https://api.telegram.org/bot{}/deleteMessage", token)
}

fn url_edit_message_text(token: &String) -> String /* dyn reqwest::IntoUrl */ {
  format!("https://api.telegram.org/bot{}/editMessageText", token)
}

fn url_send_video(token: &String) -> String /* dyn reqwest::IntoUrl */ {
  format!("https://api.telegram.org/bot{}/sendVideo", token)
}

fn url_send_audio(token: &String) -> String /* dyn reqwest::IntoUrl */ {
  format!("https://api.telegram.org/bot{}/sendAudio", token)
}


use crate::telegram_messages as messages;

#[derive(Debug, Clone)]
pub struct IncomeMessage {
  pub chat_id: i64,
  pub username: String,
  pub text: String,
}

impl std::fmt::Display for IncomeMessage {
  fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}({}): {}", self.username, self.chat_id,
           self.text.replace("\n", "<NL>"))
  }
}

/// Return (Option<update_id>, vec![IncomeMessage])
pub async fn get_updates(
  token: &String, offset: Option<i64>)
  -> Result<(Option<i64>, Vec<IncomeMessage>)> {
  let url = url_get_updates(token);
  // json::<serde_json::Value>
  let request = reqwest::Client::new().get(url);
  let request = match offset {
    None => request,
    Some(offset) => request.query(&[("offset", offset)]),
  };
  let res = request.send().await?;
  let data = res.bytes().await?;
  // log::debug!("called get_updates: parsing respnse: {:#?}", &data);
  let res1 = serde_json::from_slice::<serde_json::Value>(&data)?;
  // log::debug!("GetUpdates: {:#?}", res1);
  let res = serde_json::from_slice::<messages::GetUpdates>(&data)
    .context(format!("Could not parse GetUpdates request: {:#?}", res1))?;
  // use messages;
  use messages::{Message, Chat};
  let update_id = res.max_update_id();
  let t2 = res.to_messages();
  let t2 = t2.iter().filter_map(
    |Message {text, chat: Chat {id, username, ..}, ..}|
    if let Some(text) = text {
      Some(IncomeMessage {chat_id:*id, username:username.clone(), text: text.clone()})
    } else {
      None
    })
    .collect::<Vec<_>>();
  // log::debug!("{:#?}", t2);
  Ok((update_id, t2))
}

pub async fn send_message(
  token: &String, chat_id: i64, text: String)
  -> Result<messages::SendMessageResponse> {
  log::info!("Send to {}: {}", chat_id, &text);
  let url = url_send_message(token);
  let data = messages::SendMessage {chat_id, text, disable_notification: false, disable_web_page_preview: true};
  let client = reqwest::Client::new();
  let res = client.post(url).json(&data).send().await?;
  // let res = res.json::<serde_json::Value>().await?;
  let data = res.bytes().await?;
  log::debug!("DBG: {:#?}", &data);
  let res = serde_json::from_slice::<messages::SendMessageResponse>(&data)?;
  // let res = res.json::<messages::SendMessageResponse>().await?;
  log::debug!("{}", res);
  
  Ok(res)
}

pub async fn delete_message(
  token: &String, chat_id: i64, message_id: i64)
  -> Result<()> {
  let url = url_delete_message(token);
  let data = messages::DeleteMessage {chat_id, message_id};
  let client = reqwest::Client::new();
  let res = client.post(url).json(&data).send().await?;
  let res = res.json::<serde_json::Value>().await?;
  // let res = res.json::<messages::SendMessageResponse>().await?;
  log::debug!("delete response for {}: {}", message_id, res);
  
  Ok(())
}

pub async fn edit_message_text(
  token: &String, chat_id: i64, message_id: i64, text: String)
  -> Result<messages::SendMessageResponse> {
  // log::info!("Edit for {}: {}", chat_id, &text);
  let url = url_edit_message_text(token);
  let data = messages::EditMessageText {chat_id, message_id, text, disable_web_page_preview: true};
  let client = reqwest::Client::new();
  let res = client.post(url).json(&data).send().await?;
  // let res = res.json::<serde_json::Value>().await?;
  let res = res.json::<messages::SendMessageResponse>().await?;
  log::debug!("{}", res);
  
  Ok(res)
}

pub async fn send_video(
  token: &String, chat_id: i64, caption: String, video: String)
  -> Result<()> {
  log::info!("Send video to {}: {}", chat_id, video);
  let url = url_send_video(token);
  let request = reqwest::Client::new().post(url).query(&[
    ("chat_id", chat_id.to_string()),
    ("caption", caption),
  ]);
  let file = tokio::fs::File::open(&video).await?;
  let stream = tokio_util::codec::FramedRead::new(
    file, tokio_util::codec::BytesCodec::new());
  use reqwest::multipart::{Part, Form};
  let filename_ext = video.split(".").last().unwrap_or("mp4");
  let mime = format!("video/{}", filename_ext);
  let part = Part::stream(reqwest::Body::wrap_stream(stream))
    .file_name(video.clone())
    .mime_str(mime.as_str())?;
  // todo: use libmagic to set mime type
  let data = Form::new().part("video", part);
  let res = request.multipart(data).send().await?;
  // let res = res.json::<serde_json::Value>().await?;
  let res = res.json::<messages::SendMessageResponse>().await
    .context("Could not parse sendVideo response")?;
  log::debug!("{}", res);
  if !res.is_ok() {
    return Err(anyhow!("Could not send Video: {}", res.description));
  }
  
  Ok(())
}

pub async fn send_audio(
  token: &String, chat_id: i64, caption: String, audio: String)
  -> Result<()> {
  log::info!("Send audio to {}: {}", chat_id, audio);
  let url = url_send_audio(token);
  let request = reqwest::Client::new().post(url).query(&[
    ("chat_id", chat_id.to_string()),
    ("caption", caption),
  ]);
  let file = tokio::fs::File::open(audio.clone()).await?;
  let stream = tokio_util::codec::FramedRead::new(
    file, tokio_util::codec::BytesCodec::new());
  use reqwest::multipart::{Part, Form};
  let part = Part::stream(reqwest::Body::wrap_stream(stream))
    .file_name(audio);
    // .mime_str(format!("audio/{}", ext).as_str())?;
  // todo: use libmagic to set mime type
  let data = Form::new().part("audio", part);
  let res = request.multipart(data).send().await?;
  // let res = res.json::<serde_json::Value>().await?;
  // {"description":"Request Entity Too Large","error_code":413,"ok":false}
  let res = res.json::<messages::SendMessageResponse>().await
    .context("Could not parse sendAudio response")?;
  log::debug!("{}", res);
  
  Ok(())
}
