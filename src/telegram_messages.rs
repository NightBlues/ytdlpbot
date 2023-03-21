use std::fmt;
use serde::{Deserialize, Serialize};
// use chrono::{DateTime, Utc, NaiveDateTime};

// impl<'de> serde::de::Visitor<'de> for DateTime<Utc> {
//   type Value = i64;
//   fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//     formatter.write_str("an integer between -2^31 and 2^31")
//   }

//   fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
//   where
//     E: serde::de::Error,
//   {
//     Ok(value)
//   }

// }

// fn datetime_of_timestamp<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
// where D: serde::Deserializer<'de> {
//   let x = serde::de::value::I64Deserializer::<D::Error>::from(d);
//   // serde::de::Visitor
//   x.
//   let ts = d.deserialize_i64()?;
//   // ts: i64
//   Ok(DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ts, 0), Utc))
// }

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Chat {
  pub first_name: String,
  pub id: i64,
  #[serde(default)]
  pub last_name: String,
  #[serde(rename="type")]
  pub typ: String,
  pub username: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct From {
  pub first_name: String,
  pub id: i64,
  pub is_bot: bool,
  #[serde(default)]
  pub language_code: String,
  #[serde(default)]
  pub last_name: String,
  pub username: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
  pub message_id: i64,
  pub text: Option<String>,
  // #[serde(deserialize_with="datetime_of_timestamp")]
  // date: DateTime<Utc>,
  pub date: i64,
  pub chat: Chat,
  pub from: From,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateMessage {
  pub message: Option<Message>,
  pub update_id: i64
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetUpdates {
  pub ok: bool,
  pub result: Vec<UpdateMessage>,
}

impl GetUpdates {
  pub fn to_messages(&self) -> Vec<Message> {
    self.result.iter().filter_map(|UpdateMessage {message, ..}|
                                  if let Some(Message {text: Some(_), ..}) = message {
                                    message.clone()
                                  } else { None }).collect()
  }

  pub fn max_update_id(&self) -> Option<i64> {
    self.result.iter().map(
      |UpdateMessage {update_id, ..}| *update_id).max()
  }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct SendMessage {
  pub chat_id: i64,
  pub text: String,
  pub disable_notification: bool,
  pub disable_web_page_preview: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteMessage {
  pub chat_id: i64,
  pub message_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditMessageText {
  pub chat_id: i64,
  pub message_id: i64,
  pub text: String,
  pub disable_web_page_preview: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Thumb {
  pub file_id: String,
  pub file_size: i64,
  pub file_unique_id: String,
  pub height: i64,
  pub width: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Video {
  pub duration: i64,
  pub file_id: String,
  pub file_name: String,
  pub file_size: i64,
  pub file_unique_id: String,
  pub height: i64,
  pub mime_type: String,
  pub thumb: Option<Thumb>,
  pub width: i64,
}

impl fmt::Display for Video {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Video {} {}sec", self.file_id, self.duration)
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Audio {
  pub duration: i64,
  pub file_id: String,
  pub file_name: String,
  pub file_size: i64,
  pub file_unique_id: String,
  pub mime_type: String,
}

impl fmt::Display for Audio {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Audio {} {}sec", self.file_id, self.duration)
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SendMessageResponseInner {
  pub message_id: i64,
  pub chat: Chat,
  pub from: From,
  #[serde(default)]
  pub text: Option<String>,
  #[serde(default)]
  pub video: Option<Video>,
  #[serde(default)]
  pub audio: Option<Audio>,
}

impl fmt::Display for SendMessageResponseInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let content = match self {
      SendMessageResponseInner {text: Some(text), ..} => text.clone(),
      SendMessageResponseInner {video: Some(video), ..} => video.to_string(),
      SendMessageResponseInner {audio: Some(audio), ..} => audio.to_string(),
      _ => "not implemented".to_string(),
    };
    write!(f, "Sent message {} to {}: {}", self.message_id, self.chat.username, content)
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SendMessageResponse {
  pub ok: bool,
  pub result: SendMessageResponseInner,
}

impl fmt::Display for SendMessageResponse {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.result)
  }
}
