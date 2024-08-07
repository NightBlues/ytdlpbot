use anyhow::{Result, anyhow};
use itertools::Itertools;
use crate::telegram;
use telegram::IncomeMessage;
use crate::ytdlp;
use crate::user_state::{State, Mode, Quality, UserConfig};
use crate::config::Config;
use crate::format_chooser::{ChosenFormat, choose_format};

// Handle download command
async fn download_url_inner(conf: &Config, state: &State, chat_id: i64, url: url::Url, message_id: i64) -> Result<()> {
  let video = ytdlp::describe(url.clone()).await?;
  // log::debug!("{}", video);
  let userconf = state.get_userconfig(chat_id).await;
  let ChosenFormat {format_id, ext, vcodec, acodec} =
    choose_format(conf, &userconf, &video)?;
  telegram::edit_message_text(
    &conf.telegram_token, chat_id, message_id,
    format!("Downloading {} with format {}, video codec {:?}, audio codec {:?}...", 
            url, ext, vcodec.as_deref().unwrap_or_default(), acodec.as_deref().unwrap_or_default())).await?;
  
  // let filename = uuid::Uuid::new_v4().to_string();
  let filename = format!("{}_{}", chat_id, &video.id);
  let expected_filename = format!("{}/{}", conf.download_dir, filename);
  ytdlp::download(url.clone(), expected_filename, format_id).await?;
  let mut full_filename = None;
  let files = std::fs::read_dir(&conf.download_dir)?;
  for file in files {
    let file = file?;
    let cur_filename = file.file_name().into_string()
      .map_err(|e| anyhow!("filename not valid: {:?}", e))?;
    if cur_filename.starts_with(&filename) {
      full_filename = file.path().into_os_string().into_string().ok();
    }
  }
  let full_filename = full_filename.ok_or(anyhow!("Could not find downloaded file"))?;
  match userconf.mode {
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

/// Download URL, reporting error back to chat
async fn download_url(conf: &Config, state: &State, msg: &IncomeMessage, url: url::Url) -> Result<()> {
  let &IncomeMessage {chat_id, ..} = msg;
  let response = telegram::send_message(
    &conf.telegram_token, chat_id,
    format!("Downloading {}...", url)).await?;
  let result = response.result.ok_or(anyhow!(response.description))?;
  let message_id = result.message_id;
  let res = download_url_inner(conf, state, chat_id, url, message_id).await;
  if let Err(e) = &res {
    telegram::edit_message_text(&conf.telegram_token, chat_id, message_id, e.to_string()).await?;
  };
  
  Ok(())
}


// Dispatch commands
pub async fn react(conf: &Config, state: &State, msg: &IncomeMessage) -> Result<()> {
  log::info!("command {}", msg);
  match url::Url::parse(&msg.text) {
    Ok(url) => {
      let res = download_url(conf, state, msg, url).await;
      match res {
        Ok(()) => (),
        Err(e) => log::error!("Error: {:?}", e),
      }
          
      Ok(())
    },
    Err(_) => {
      let &IncomeMessage {chat_id, ..} = msg;
      let words : Vec<_> = msg.text.split_whitespace()
      // .map(|x| x.to_string())
        .collect();
      match words.as_slice() {
        ["/st", ..] => {
          let userconf = state.get_userconfig(chat_id).await;
          telegram::send_message(
            &conf.telegram_token, chat_id,
            format!("Current user config is:\n{}", userconf)).await?;
          Ok(())
        },
        ["/audio", ..] => {
          state.set_mode(chat_id, Mode::Audio).await;
          telegram::send_message(
            &conf.telegram_token, chat_id,
            "Switched to audio download".to_string()).await?;
          Ok(())
        },
        ["/video", ..] => {
          state.set_mode(chat_id, Mode::Video).await;
          telegram::send_message(
            &conf.telegram_token, chat_id,
            "Switched to video download".to_string()).await?;
          Ok(())
        },
        ["/video_quality_high", ..] => {
          state.set_video_quality(chat_id, Quality::High).await;
          telegram::send_message(
            &conf.telegram_token, chat_id,
            "Set video quality to High".to_string()).await?;
          Ok(())
        },
        ["/video_quality_low", ..] => {
          state.set_video_quality(chat_id, Quality::Low).await;
          telegram::send_message(
            &conf.telegram_token, chat_id,
            "Set video quality to Low".to_string()).await?;
          Ok(())
        },
        ["/audio_quality_high", ..] => {
          state.set_audio_quality(chat_id, Quality::High).await;
          telegram::send_message(
            &conf.telegram_token, chat_id,
            "Set audio quality to High".to_string()).await?;
          Ok(())
        },
        ["/audio_quality_low", ..] => {
          state.set_audio_quality(chat_id, Quality::Low).await;
          telegram::send_message(
            &conf.telegram_token, chat_id,
            "Set audio quality to Low".to_string()).await?;
          Ok(())
        },
        ["/vcodec_exclude", vcodecs @ ..] => {
          let vcodecs = vcodecs.iter().map(|x| x.to_string()).collect();
          let UserConfig {vcodec_exclude, .. } =
            state.set_vcodec_exclude(chat_id, vcodecs).await;
          let msg = format!("Set video codecs excludes to {}",
                            vcodec_exclude.join(" "));
          telegram::send_message(
            &conf.telegram_token, chat_id, msg).await?;
          Ok(())
        },
        _ =>  {
          telegram::send_message(
            &conf.telegram_token, chat_id,
            "Unknown command".to_string()).await?;
          Ok(())
        }
      }
    }
  }
}


// Throttle and call dispatcher
pub async fn react_messages(conf: &Config, state: &State, messages: Vec<IncomeMessage>) -> Result<()> {
  let messages = messages.iter()
    .sorted_by_key(|x| &x.username)
    .group_by(|x| &x.username);
  for (username, group) in &messages {
    // if group.collect().
    let group = group.collect_vec();
    match group[..] {
      [] => continue,
      [msg] => react(conf, state, msg).await?,
      [IncomeMessage {chat_id, ..}, ..] => {
        log::warn!("User {} Too many requests", username);
        telegram::send_message(
          &conf.telegram_token, *chat_id,
          "Too many requests".to_string()).await?;
      }
    }
  }

  Ok(())
}
