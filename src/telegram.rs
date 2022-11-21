// use std::collections::hash_map::HashMap;
use std::vec::Vec;
use anyhow::{Result, Context};


#[derive(Clone)]
pub struct Conf {
  pub token: String,
}

impl Conf {
  fn url_get_updates(&self) -> String /* dyn reqwest::IntoUrl */ {
    format!("https://api.telegram.org/bot{}/getUpdates", self.token)
  }
  fn url_send_message(&self) -> String /* dyn reqwest::IntoUrl */ {
    format!("https://api.telegram.org/bot{}/sendMessage", self.token)
  }
  fn url_send_video(&self) -> String /* dyn reqwest::IntoUrl */ {
    format!("https://api.telegram.org/bot{}/sendVideo", self.token)
  }
}


use crate::telegram_messages as messages;


/// Return (Option<update_id>, vec![(chat_id, username, text)])
pub async fn get_updates(
  conf: Conf, offset: Option<i64>)
  -> Result<(Option<i64>, Vec<(i64, String, String)>)> {
  let url = conf.url_get_updates();
  // json::<serde_json::Value>
  let request = reqwest::Client::new().get(url);
  let request = match offset {
    None => request,
    Some(offset) => request.query(&[("offset", offset)]),
  };
  let res = request.send().await?;
  let data = res.bytes().await?;
  let res1 = serde_json::from_slice::<serde_json::Value>(&data)?;
  // println!("GetUpdates: {:#?}", res1);
  let res = serde_json::from_slice::<messages::GetUpdates>(&data)
    .context(format!("Could not parse GetUpdates request: {:#?}", res1))?;
  // use messages;
  use messages::{Message, Chat};
  let update_id = res.max_update_id();
  let t2 = res.to_messages();
  let t2 = t2.iter().filter_map(
    |Message {text, chat: Chat {id, username, ..}, ..}|
    if let Some(text) = text {
      Some((*id, username.clone(), text.clone()))
    } else {
      None
    })
    .collect::<Vec<_>>();
  // println!("{:#?}", t2);
  Ok((update_id, t2))
}

pub async fn send_message(
  conf: Conf, chat_id: i64, text: String)
  -> Result<()> {
  let url = conf.url_send_message();
  let data = messages::SendMessage {chat_id, text, disable_notification: false};
  let client = reqwest::Client::new();
  let res = client.post(url).json(&data).send().await?;
  // let res = res.json::<serde_json::Value>().await?;
  let res = res.json::<messages::SendMessageResponse>().await?;
  println!("{}", res);
  
  Ok(())
}

pub async fn send_video(
  conf: Conf, chat_id: i64, caption: String, video: String)
  -> Result<()> {
  let url = conf.url_send_video();
  let request = reqwest::Client::new().post(url).query(&[
    ("chat_id", chat_id.to_string()),
    ("caption", caption),
  ]);
  let file = tokio::fs::File::open(video).await?;
  let stream = tokio_util::codec::FramedRead::new(
    file, tokio_util::codec::BytesCodec::new());
  use reqwest::multipart::{Part, Form};
  let part = Part::stream(reqwest::Body::wrap_stream(stream))
    .file_name("test.mp4")
    .mime_str("video/mp4")?;
  // todo: use libmagic to set mime type
  let data = Form::new().part("video", part);
  let res = request.multipart(data).send().await?;
  // let res = res.json::<serde_json::Value>().await?;
  let res = res.json::<messages::SendMessageResponse>().await
    .context("Could not parse sendVideo response")?;
  println!("{}", res);
  
  Ok(())
}
