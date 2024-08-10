use tokio::sync::RwLock;
use anyhow::{Result};
// use itertools::Itertools;
use lru::LruCache;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
  Video,
  Audio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quality {
  Low,
  High,
  Awful,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutInterval {
  pub start: i64,
  pub end: i64,  
}

impl CutInterval {
  pub fn parse(s: &str, e: &str) -> Result<Self> {
    let start = s.parse::<i64>()?;
    let end = e.parse::<i64>()?;
    Ok(CutInterval {start, end})
  }
}

impl std::fmt::Display for CutInterval {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserConfig {
  pub mode: Mode,
  pub aquality: Quality,
  pub vquality: Quality,
  pub vcodec_exclude: Vec<String>,
  pub cut_interval: Option<CutInterval>,
}

impl std::fmt::Display for UserConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let UserConfig {mode, aquality, vquality, vcodec_exclude,
                    cut_interval} = (*self).clone();
    let vcodecs = vcodec_exclude.join(",");
    let cut_interval = match cut_interval {
      None => String::new(),
      Some(i) => format!("{}", i)
    };
    write!(f, "download mode: {:?}\naudio quality: {:?}\nvideo quality: {:?}\nvideo codecs excluded: {}\ncut interval: {}\n",
           mode, aquality, vquality, vcodecs, cut_interval)
  }
}

impl UserConfig {
  pub fn new() -> UserConfig {
    UserConfig {mode: Mode::Video, aquality: Quality::Low, vquality: Quality::Low, vcodec_exclude: vec![], cut_interval: None}
  }
}

impl Default for UserConfig {
  fn default() -> Self {
    UserConfig::new()
  }
}

pub struct State {
  pub configs: RwLock<LruCache<i64, UserConfig>>,
}

impl State {
  pub fn new() -> State {
    let configs = RwLock::new(LruCache::new(std::num::NonZeroUsize::new(100).unwrap()));
    State {configs}
  }

  pub async fn get_userconfig(self: &State, chat_id: i64) -> UserConfig {
    let configs = self.configs.read().await;
    let val = configs.peek(&chat_id)
      .map(|x| (*x).clone())
      .unwrap_or_default();

    val
  }

  pub async fn set_userconfig(self: &State, chat_id: i64, val: UserConfig) -> UserConfig {
    let mut config = self.configs.write().await;
    config.put(chat_id, val.clone());
    val
  }

  pub async fn get_mode(self: &State, chat_id: i64) -> Mode {
    let UserConfig {mode, ..} = self.get_userconfig(chat_id).await;
    mode
  }

  /// Helper function to atomically update userstate.
  pub async fn update_userconfig<F>(&self, chat_id: i64, f: F) -> UserConfig
  where F: FnOnce(UserConfig) -> UserConfig
  {
    let mut config = self.configs.write().await;
    let val = config.peek(&chat_id)
      .map(|x| (*x).clone())
      .unwrap_or_default();
    let val = f(val);
    config.put(chat_id, val.clone());
    val
  }

  pub async fn set_mode(self: &State, chat_id: i64, mode: Mode) -> UserConfig {
    self.update_userconfig(chat_id, move |val| UserConfig {mode, .. val}).await
  }

  pub async fn set_video_quality(self: &State, chat_id: i64, vquality: Quality) -> UserConfig {
    self.update_userconfig(chat_id,
                           |val| UserConfig {vquality, .. val}).await
  }

  pub async fn set_audio_quality(self: &State, chat_id: i64, aquality: Quality) -> UserConfig {
    self.update_userconfig(chat_id,
                           |val| UserConfig {aquality, .. val}).await
  }

  pub async fn set_vcodec_exclude(self: &State, chat_id: i64, vcodec_exclude: Vec<String>) -> UserConfig {
    self.update_userconfig(chat_id,
                           |val| UserConfig {vcodec_exclude, .. val}).await
  }

  pub async fn set_cut_inteval(self: &State, chat_id: i64, cut_interval: Option<CutInterval>) -> UserConfig {
    self.update_userconfig(chat_id,
                           |val| UserConfig {cut_interval, .. val}).await
  }


}
