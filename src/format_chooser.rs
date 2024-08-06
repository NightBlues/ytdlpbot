use anyhow::{Result, anyhow};
use itertools::Itertools;
use crate::config::Config;
use crate::ytdlp;
use crate::user_state::{UserConfig, Quality, Mode};


pub struct ChosenFormat {
  pub format_id: Option<String>,
  pub ext: String,
  pub vcodec: Option<String>,
  pub acodec: Option<String>,
}

fn choose_format_audio(conf: &Config, userconf: &UserConfig, video: &ytdlp::Video) -> Result<ChosenFormat> {
  let Config {max_filesize, ..} = conf.clone();
  let UserConfig {aquality, ..} = userconf.clone();
  // println!("max_filesize={}, vcodec={:?}", max_filesize, &video.vcodec);
  println!("DBG: All formats: {}", ytdlp::FormatVec(video.formats.clone()));
  use ytdlp::Format;
  let mut formats : Vec<_> = video.formats.iter()
    .sorted_by_key(|x| x.get_filesize().unwrap_or(max_filesize))
    .filter(|x| x.get_filesize()
            .map_or(false, |filesize| filesize < max_filesize))
    .filter(|format| {
      let (video, audio) = format.get_video_audio();
      println!("DBG: {:?} {}", video.clone(), audio.clone());
      video == "none" && audio != "none"
    })
    .rev()
    .collect();
  if aquality == Quality::Low {
    formats.sort_by_key(|Format {tbr, ..}| tbr.clone().map(|x| -x as i64));
  } else {
    formats.sort_by_key(|Format {tbr, ..}| tbr.clone().map(|x| x as i64));
  }
  if formats.is_empty() {
    Err(anyhow!("Sorry, file is too big"))
  } else {
    let Format {format_id, ext, vcodec, acodec, video_ext, audio_ext, ..} = formats[0];
    println!("Chosen vcodec: {:?}, acodec: {:?}, video_ext: {:?}, audio_ext: {:?}", vcodec, acodec, video_ext, audio_ext);
    Ok(ChosenFormat {
      format_id: Some(format_id.clone()),
      ext: ext.clone(),
      vcodec: vcodec.clone(),
      acodec: acodec.clone()})
  }
}


fn choose_format_video(conf: &Config, userconf: &UserConfig, video: &ytdlp::Video) -> Result<ChosenFormat> {
  let Config {max_filesize, ..} = conf.clone();
  let UserConfig {vquality, vcodec_exclude, ..} = userconf.clone();
  println!("DBG: All formats: {}", ytdlp::FormatVec(video.formats.clone()));
  use ytdlp::Format;
  let mut formats : Vec<_> = video.formats.iter()
    .filter(|x| x.get_filesize()
            .map_or(false, |filesize| filesize < max_filesize))
    .filter(|format| {
      let (video, audio) = format.get_video_audio();
      println!("DBG: {:?} {}", video.clone(), audio.clone());
      let excluded = vcodec_exclude.iter()
        .find(|&c| video.starts_with(c)).is_some();
      video != "none" && audio != "none" && !excluded
    })
    .filter(|format| {
      // exclude too shitty resolutions
      format.height.unwrap_or(0) >= 360
    })
    .sorted_by_key(|x| x.get_filesize().unwrap_or(max_filesize))
    .rev()
    .collect();
  if vquality == Quality::Low {
    
    formats.sort_by_key(|Format {tbr, ..}| tbr.clone().map(|x| -x as i64));
  } else {
    formats.sort_by_key(|Format {tbr, ..}| tbr.clone().map(|x| x as i64));
  }
  if formats.is_empty() {
    Err(anyhow!("Sorry, file is too big or you have exluded all formats"))
  } else {
    let Format {format_id, ext, vcodec, acodec, video_ext, audio_ext, ..} = formats[0];
    println!("Chosen vcodec: {:?}, acodec: {:?}, video_ext: {:?}, audio_ext: {:?}", vcodec, acodec, video_ext, audio_ext);
    Ok(ChosenFormat {
      format_id: Some(format_id.clone()),
      ext: ext.clone(),
      vcodec: vcodec.clone(),
      acodec: acodec.clone()})
  }
}


pub fn choose_format(conf: &Config, userconf: &UserConfig, video: &ytdlp::Video) -> Result<ChosenFormat> {
  match userconf {
    UserConfig {mode: Mode::Video, ..} =>
      choose_format_video(conf, userconf, video),
    UserConfig {mode: Mode::Audio, ..} =>
      choose_format_audio(conf, userconf, video),
  }
}
