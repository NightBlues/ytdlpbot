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

impl From<ytdlp::Format> for ChosenFormat {
  fn from(format: ytdlp::Format) -> ChosenFormat {
    let ytdlp::Format {format_id, ext, vcodec, acodec, ..} = format;
    ChosenFormat {
      format_id: Some(format_id.clone()),
      ext: ext.clone(),
      vcodec: vcodec.clone(),
      acodec: acodec.clone()}
  }
}

fn choose_format_audio(conf: &Config, userconf: &UserConfig, video: &ytdlp::Video) -> Result<ytdlp::Format> {
  let Config {max_filesize, ..} = conf.clone();
  let UserConfig {aquality, ..} = userconf.clone();
  // log::info!("max_filesize={}, vcodec={:?}", max_filesize, &video.vcodec);
  use ytdlp::Format;
  let mut formats : Vec<ytdlp::Format> = video.formats.clone()
    .into_iter()
    .sorted_by_key(|x| x.get_filesize().unwrap_or(max_filesize))
    .filter(|x| x.get_filesize()
            .map_or(false, |filesize| filesize < max_filesize))
    .filter(|format| {
      let (video, audio) = format.get_video_audio();
      // log::debug!("DBG: {:?} {}", video.clone(), audio.clone());
      video == "none" && audio != "none"
    })
    .filter(|format| {
      // exclude too shitty bitrates
      format.tbr.unwrap_or(0.0) >= 49.0
    })
    .rev()
    .collect();
  match aquality {
    Quality::High =>
      formats.sort_by_key(|Format {tbr, ..}| tbr.map(|x| -x as i64)),
    _ =>
      formats.sort_by_key(|Format {tbr, ..}| tbr.map(|x| x as i64)),
  };
  log::debug!("DBG: Filtered audio formats: {}", ytdlp::FormatVec(formats.clone()));
  if formats.is_empty() {
    Err(anyhow!("Sorry, file is too big"))
  } else {
    // let Format {vcodec, acodec, video_ext, audio_ext, ..} = formats[0];
    // log::info("Chosen vcodec: {:?}, acodec: {:?}, video_ext: {:?}, audio_ext: {:?}", vcodec, acodec, video_ext, audio_ext);
    Ok(formats[0].clone())
  }
}


fn choose_format_video(conf: &Config, userconf: &UserConfig, video: &ytdlp::Video) -> Result<ytdlp::Format> {
  let Config {max_filesize, ..} = conf.clone();
  let UserConfig {vquality, vcodec_exclude, ..} = userconf.clone();
  use ytdlp::Format;
  let audio_format = choose_format_audio(conf, userconf, video).ok();
  let mut formats : Vec<_> = video.formats.iter()
    .filter(|x| x.get_filesize()
            .map_or(false, |filesize| filesize < max_filesize))
    .filter_map(|format| {
      let (video, audio) = format.get_video_audio();
      // log::debug!("DBG: {:?} {}", video.clone(), audio.clone());
      let excluded = vcodec_exclude.iter()
        .any(|c| video.starts_with(c));
      match &(&*video, &*audio, excluded) {
        // if not video or is excluded
        ("none", _, _) => None,
        (_, _, true) => None,
        // if video only - transfer it to video+audio
        (_, "none", false) => {
          audio_format.as_ref().map(
            |audio_format| format.add_audio(audio_format))
        },
        // if video + audio and not exluded
        (_, _, false) => Some(format.clone()),
      }
    })
    // apply filsize filter again after posible merging with audio
    .filter(|x| x.get_filesize()
            .map_or(false, |filesize| filesize < max_filesize))
    .filter(|format| {
      // exclude too shitty resolutions if not Awful
      vquality == Quality::Awful || format.height.unwrap_or(0) >= 360
    })
    .sorted_by_key(|x| x.get_filesize().unwrap_or(max_filesize))
    .rev()
    .collect();
  match vquality {
    Quality::High => {
      formats.sort_by_key(|Format {tbr, ..}| tbr.map(|x| -x as i64));
    },
    _ => {
      formats.sort_by_key(|Format {tbr, ..}| tbr.map(|x| x as i64));
    },
  };
  log::debug!("DBG: Filtered video formats: {}", ytdlp::FormatVec(formats.clone()));
  if formats.is_empty() {
    Err(anyhow!("Sorry, file is too big or you have exluded all formats"))
  } else {
    // let Format {vcodec, acodec, video_ext, audio_ext, ..} = formats[0];
    // log::info("Chosen vcodec: {:?}, acodec: {:?}, video_ext: {:?}, audio_ext: {:?}", vcodec, acodec, video_ext, audio_ext);
    Ok(formats[0].clone())
  }
}


pub fn choose_format(conf: &Config, userconf: &UserConfig, video: &ytdlp::Video) -> Result<ChosenFormat> {
  log::debug!("DBG: All formats: {}", ytdlp::FormatVec(video.formats.clone()));
  let res = 
    match userconf {
      UserConfig {mode: Mode::Video, ..} =>
        choose_format_video(conf, userconf, video),
      UserConfig {mode: Mode::Audio, ..} => {
        choose_format_audio(conf, userconf, video)
      }
    }?;
  Ok(res.into())
}
