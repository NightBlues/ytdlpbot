use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use itertools::Itertools;
use lru::LruCache;
use crate::telegram;
use crate::ytdlp;

#[derive(Clone)]
pub struct Config {
  pub max_filesize: i64,
  pub vcodec_exclude: Vec<String>,
  pub telegram_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
  Video,
  Audio,
}

pub struct State {
  pub modes: RwLock<LruCache<i64, Mode>>,
}

impl State {
  pub fn new() -> State {
    let modes = RwLock::new(LruCache::new(std::num::NonZeroUsize::new(100).unwrap()));
    State {modes}
  }

  pub async fn get_mode(self: &State, chat_id: i64) -> Mode {
    let modes = self.modes.read().await;
    let val = modes.peek(&chat_id).unwrap_or(&Mode::Video);

    (*val).clone()
  }
}

/// Return (Option<format_id>, ext)
fn choose_format(conf: &Config, mode: Mode, video: &ytdlp::Video) -> Result<(Option<String>, String)> {
  let Config {max_filesize, vcodec_exclude, ..} = conf.clone();
  println!("max_filesize={}", max_filesize);
  let filsize : i64 = video.filesize_approx.unwrap_or(max_filesize);
  if filsize < max_filesize && !vcodec_exclude.contains(&video.vcodec) && mode == Mode::Video {
    return Ok((None, video.ext.clone()))
  }
  use ytdlp::Format;
  let formats : Vec<_> = video.formats.iter()
    .sorted_by_key(|x| x.get_filesize().unwrap_or(max_filesize))
    .filter(|x @ Format {vcodec, acodec, ..} |
            match x.get_filesize() {
              None => false,
              Some(filesize) => {
                // println!("DBG: {} {}", vcodec, filesize);
                filesize < max_filesize
                  && (if mode == Mode::Video { vcodec != "none" } else {vcodec == "none" })
                  && acodec != "none"
                  && !vcodec_exclude.contains(&vcodec)
              },
            })
    .rev()
    .collect();
  // println!("Formats = {:#?}", formats);
  match formats[..] {
    [] => Err(anyhow!("Sorry, file is too big: {}", filsize)),
    [Format {format_id, ext, vcodec, acodec, ..}, ..] => {
      println!("Chosen vcodec: {}, acodec: {}", vcodec, acodec);
      Ok((Some(format_id.clone()), ext.clone()))
    },
  }
}


async fn download_url(conf: &Config, state: &State, chat_id: i64, url: url::Url) -> Result<()> {
  let response = telegram::send_message(
    &conf.telegram_token, chat_id,
    format!("Downloading {}...", url)).await?;
  let message_id = response.result.message_id;
  // telegram::send_message(conf.clone(), chat_id, text).await?
  // let url = "https://youtu.be/kseKKaa94vg".to_string();
  let video = ytdlp::describe(url.clone()).await?;
  // println!("{:#?}", video);
  println!("{}", video);
  let mode = state.get_mode(chat_id).await;
  let (format_id, ext) =
    match choose_format(conf, mode.clone(), &video) {
      Ok(x) => x,
      Err(e) => {
        telegram::edit_message_text(&conf.telegram_token, chat_id, message_id, e.to_string()).await?;
        return Ok(())
      }
    };
  telegram::edit_message_text(
    &conf.telegram_token, chat_id, message_id,
    format!("Downloading {} with format {}...", url, ext)).await?;
 
  // let filename = uuid::Uuid::new_v4().to_string();
  let filename = video.id;
  let full_filename = format!("{}.{}", &filename, ext.clone());
  let filename_template = format!("{}.%(ext)s", &filename);
  ytdlp::download(url.clone(), filename_template, format_id).await?;
  match mode {
    Mode::Video =>
      telegram::send_video(&conf.telegram_token, chat_id, video.title.clone(), full_filename.clone()).await?,
    Mode::Audio =>
      telegram::send_audio(&conf.telegram_token, chat_id, video.title.clone(), full_filename.clone()).await?,
  };
  std::fs::remove_file(full_filename)?;
  telegram::delete_message(
    &conf.telegram_token, chat_id, message_id).await?;
  Ok(())
}

pub async fn react(conf: &Config, state: &State, chat_id: i64, text: String) -> Result<()> {
  match url::Url::parse(&text) {
    Ok(url) => {
      let res = download_url(conf, state, chat_id, url).await;
      match res {
        Ok(()) => (),
        Err(e) => println!("Error: {:?}", e),
      }
          
      return Ok(())
    },
    Err(_) =>
      if text.starts_with("/st") {
        let mode = state.get_mode(chat_id).await;
        telegram::send_message(
          &conf.telegram_token, chat_id,
          format!("Current mode is: {:?}", mode)).await?;
        return Ok(())
      } else if text.starts_with("/audio") {
        let mut modes = state.modes.write().await;
        modes.put(chat_id, Mode::Audio);
        telegram::send_message(
          &conf.telegram_token, chat_id,
          "Switched to audio download".to_string()).await?;
        return Ok(())
      } else if text.starts_with("/video") {
        let mut modes = state.modes.write().await;
        modes.put(chat_id, Mode::Video);
        telegram::send_message(
          &conf.telegram_token, chat_id,
          "Switched to video download".to_string()).await?;
        return Ok(())
      }
  }
  telegram::send_message(
    &conf.telegram_token, chat_id,
    "Unknown command".to_string()).await?;

  Ok(())
}

pub async fn react_messages(conf: &Config, state: &State, messages: Vec<(i64, String, String)>) -> Result<()> {
    let messages = messages.iter()
      .sorted_by_key(|x| x.1.clone())
      .group_by(|(_, x, _)| x);
    for (username, group) in &messages {
      // if group.collect().
      let group = group.collect_vec();
      match group[..] {
        [] => continue,
        [(chat_id, _, text)] =>
          react(conf, state, *chat_id, text.clone()).await?,
        [(chat_id, _, _), ..] => {
          println!("User {} Too many requests", username);
          telegram::send_message(
            &conf.telegram_token, *chat_id,
            "Too many requests".to_string()).await?;
        }
      }
    }

    Ok(())
}
