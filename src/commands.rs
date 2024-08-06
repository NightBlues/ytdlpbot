use anyhow::{Result, anyhow};
use itertools::Itertools;
use crate::telegram;
use crate::ytdlp;
use crate::user_state::{State, Mode, Quality, UserConfig};
use crate::config::Config;
use crate::format_chooser::{ChosenFormat, choose_format};

// Handle download command
async fn download_url(conf: &Config, state: &State, chat_id: i64, url: url::Url) -> Result<()> {
  let response = telegram::send_message(
    &conf.telegram_token, chat_id,
    format!("Downloading {}...", url)).await?;
  let result = response.result.ok_or(anyhow!(response.description))?;
  let message_id = result.message_id;
  // telegram::send_message(conf.clone(), chat_id, text).await?
  // let url = "https://youtu.be/kseKKaa94vg".to_string();
  let video = ytdlp::describe(url.clone()).await;
  if let Err(e) = &video {
     telegram::edit_message_text(&conf.telegram_token, chat_id, message_id, e.to_string()).await?;
  };
  let video = video?;
  // println!("{:#?}", video);
  println!("{}", video);
  let userconf = state.get_userconfig(chat_id).await;
  let ChosenFormat {format_id, ext, vcodec, acodec} =
    match choose_format(conf, &userconf, &video) {
      Ok(x) => x,
      Err(e) => {
        telegram::edit_message_text(&conf.telegram_token, chat_id, message_id, e.to_string()).await?;
        return Ok(())
      }
    };
  telegram::edit_message_text(
    &conf.telegram_token, chat_id, message_id,
    format!("Downloading {} with format {}, video codec {:?}, audio codec {:?}...", 
            url, ext, vcodec.as_deref().unwrap_or_default(), acodec.as_deref().unwrap_or_default())).await?;
  
  // let filename = uuid::Uuid::new_v4().to_string();
  let filename = video.id;
  let full_filename = format!("{}.{}", &filename, ext.clone());
  let filename_template = format!("{}.%(ext)s", &filename);
  let download_res =
    ytdlp::download(url.clone(), filename_template, format_id).await;
  if let Err(e) = &download_res {
    telegram::edit_message_text(&conf.telegram_token, chat_id, message_id, e.to_string()).await?;
    return Err(anyhow!("download error"))
  };
  let upload_res = match userconf.mode {
    Mode::Video =>
      telegram::send_video(&conf.telegram_token, chat_id, video.title.clone(), full_filename.clone()).await,
    Mode::Audio =>
      telegram::send_audio(&conf.telegram_token, chat_id, video.title.clone(), full_filename.clone()).await,
  };
  if let Err(e) = &upload_res {
    telegram::edit_message_text(&conf.telegram_token, chat_id, message_id, e.to_string()).await?;
    std::fs::remove_file(full_filename)?;
    return Err(anyhow!("download error"))
  };
  std::fs::remove_file(full_filename)?;
  telegram::delete_message(
    &conf.telegram_token, chat_id, message_id).await?;
  Ok(())
}


// Dispatch commands
pub async fn react(conf: &Config, state: &State, chat_id: i64, text: String) -> Result<()> {
  match url::Url::parse(&text) {
    Ok(url) => {
      let res = download_url(conf, state, chat_id, url).await;
      match res {
        Ok(()) => (),
        Err(e) => println!("Error: {:?}", e),
      }
          
      Ok(())
    },
    Err(_) => {
      let words : Vec<_> = text.split_whitespace()
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
