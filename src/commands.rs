use anyhow::{Result, anyhow};
use itertools::Itertools;
use crate::telegram;
use crate::ytdlp;

const MAX_FILESIZE : i64 = 50 * 1024 * 1024;

/// Return (Option<format_id>, ext)
fn choose_video_format(video: &ytdlp::Video) -> Result<(Option<String>, String)> {
  let filsize : i64 = video.filesize_approx.unwrap_or(MAX_FILESIZE);
  if filsize < MAX_FILESIZE {
    return Ok((None, video.ext.clone()))
  }
  use ytdlp::Format;
  let formats : Vec<_> = video.formats.iter()
    .sorted_by_key(|x| x.get_filesize().unwrap_or(MAX_FILESIZE))
    .filter(|x @ Format {vcodec, acodec, ..} |
            match x.get_filesize() {
              None => false,
              Some(filesize) =>
                filesize < MAX_FILESIZE && vcodec != "none" && acodec != "none",
            })
    .rev()
    .collect();
  // println!("Formats = {:#?}", formats);
  match formats[..] {
    [] => Err(anyhow!("Sorry, video is too big: {}", filsize)),
    [Format {format_id, ext, ..}, ..] => Ok((Some(format_id.clone()), ext.clone())),
  }
}


async fn download_video(conf: telegram::Conf, chat_id: i64, url: url::Url) -> Result<()> {
  // telegram::send_message(conf.clone(), chat_id, text).await?
  // let url = "https://youtu.be/kseKKaa94vg".to_string();
  let video = ytdlp::describe(url.clone()).await?;
  // println!("{:#?}", video);
  println!("{}", video);
  let (format_id, ext) = match choose_video_format(&video) {
    Ok(x) => x,
    Err(e) => {
      telegram::send_message(conf.clone(), chat_id, e.to_string()).await?;
      return Ok(())
    }
  };
  
  // let filename = uuid::Uuid::new_v4().to_string();
  let filename = video.id;
  let full_filename = format!("{}.{}", &filename, ext);
  let filename_template = format!("{}.%(ext)s", &filename);
  ytdlp::download(url.clone(), filename_template, format_id).await?;
  telegram::send_video(conf.clone(), chat_id, video.title.clone(), full_filename.clone()).await?;
  std::fs::remove_file(full_filename)?;
  Ok(())
}

pub async fn react(conf: telegram::Conf, chat_id: i64, text: String) -> Result<()> {
  match url::Url::parse(&text) {
    Ok(url) => {
      let res = download_video(conf, chat_id, url).await;
      match res {
        Ok(()) => (),
        Err(e) => println!("Error: {:?}", e),
      }
          
      return Ok(())
    },
    Err(_) =>
      if text.starts_with("/st") {
        telegram::send_message(
          conf.clone(), chat_id,
          "I'm ok, thanks!".to_string()).await?;
        return Ok(())
      }
  }

  Ok(())
}

pub async fn react_messages(conf: telegram::Conf, messages: Vec<(i64, String, String)>) -> Result<()> {
    let messages = messages.iter()
      .sorted_by_key(|x| x.1.clone())
      .group_by(|(_, x, _)| x);
    for (username, group) in &messages {
      // if group.collect().
      let group = group.collect_vec();
      match group[..] {
        [] => continue,
        [(chat_id, _, text)] =>
          react(conf.clone(), *chat_id, text.clone()).await?,
        [(chat_id, _, _), ..] => {
          println!("User {} Too many requests", username);
          telegram::send_message(
            conf.clone(), *chat_id,
            "Too many requests".to_string()).await?
        }
      }
    }

    Ok(())
}
