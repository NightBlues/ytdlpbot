use std::fmt;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Error, Context};
use tokio::process::Command;



#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Format {
  pub format_id: String,
  #[serde(default)]
  pub filesize_approx: Option<i64>,
  pub vcodec: Option<String>,
  #[serde(default)]
  pub acodec: Option<String>,
  #[serde(default)]
  pub video_ext: Option<String>,
  #[serde(default)]
  pub audio_ext: Option<String>,
  #[serde(default)]
  pub ext: String,
  pub height: Option<i64>,
  pub width: Option<i64>,
  #[serde(default)]
  pub filesize: Option<i64>,
  pub tbr: Option<f64>,
  pub abr: Option<f64>,
  pub asr: Option<f64>,
  pub vbr: Option<f64>,
  pub fps: Option<f64>,
}

impl Format {
  pub fn get_filesize(&self) -> Option<i64> {
    self.filesize.or(self.filesize_approx)
  }

  pub fn get_video_audio(&self) -> (String, String) {
    let Format {vcodec, acodec,
                audio_ext, video_ext, ..} = (*self).clone();
    let video = vcodec.or(video_ext).unwrap_or_else(|| "none".to_string());
    let audio = acodec.or(audio_ext).unwrap_or_else(|| "none".to_string());
    (video, audio)
  }

  pub fn add_audio(&self, audio: &Format) -> Self {
    let filesize_approx =
      self.filesize_approx
      .and_then(|fs| audio.filesize_approx.map(|fs_a| fs + fs_a))
      .or(self.filesize_approx);
    let filesize =
      self.filesize
      .and_then(|fs| audio.filesize.map(|fs_a| fs + fs_a))
      .or(self.filesize);
    Format {
      format_id: format!("{}+{}", self.format_id, audio.format_id),
      acodec: audio.acodec.clone(),
      audio_ext: audio.audio_ext.clone(),
      filesize_approx,
      filesize,
      abr: audio.abr,
      .. self.clone()}
  }
}

impl fmt::Display for Format {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let Format {format_id, tbr, ..} = (*self).clone();
    let (video, audio) = self.get_video_audio();
    let filesize = self.get_filesize();

    write!(f, "{{{}: {} {} {:?} {:?}}}", format_id, video, audio, tbr, filesize)
  }
}

pub struct FormatVec(pub Vec<Format>);

impl fmt::Display for FormatVec {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let format_strs: Vec<_> = self.0.clone().into_iter().map(|x| format!("{}", x)).collect();
    write!(f, "{}", format_strs.join("\n"))
  }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct Video {
  pub id: String,
  pub title: String,
  pub filename: String,
  pub ext: String,
  // #[serde(default)]
  //pub fps: f64,
  pub width: i64,
  pub height: i64,
  pub vcodec: Option<String>,
  #[serde(default)]
  pub acodec: Option<String>,
  #[serde(default)]
  pub video_ext: Option<String>,
  #[serde(default)]
  pub audio_ext: Option<String>,
  //  #[serde(default)]
  //  pub vbr: f64,
  pub format: String,
  pub format_id: String,
  pub formats: Vec<Format>,
  #[serde(default, alias="filesize")]
  pub filesize_approx: Option<i64>,
  pub duration: f64,
  pub tbr: Option<f64>,
  pub abr: Option<f64>,
  pub asr: Option<f64>,
  pub vbr: Option<f64>,
  pub fps: Option<f64>,
}

impl std::fmt::Display for Video {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Video: {}.{} {} {:?}bytes {}secs",
           self.id, self.ext, self.title, self.filesize_approx, self.duration)
  }
}


pub async fn describe(url: url::Url) -> Result<Video> {
  let mut cmd = Command::new("yt-dlp");
  cmd.arg("-j").arg(url.to_string());
  log::info!("ytdlp::describe {:?}", &cmd);
  let output = cmd.output();
  let output = output.await?;

  if !output.status.success() {
    // Err(output.stderr.to_string())
    log::error!("stdout: {:?}\nstderr: {:?}",
                std::str::from_utf8(&output.stdout).unwrap(),
                std::str::from_utf8(&output.stderr).unwrap());
    Err(Error::msg("Command describe failed"))
  } else { Ok(()) }?;

  // let _res_raw : serde_json::Value = serde_json::from_slice(&output.stdout)?;
  let result = serde_json::from_slice::<Video>(&output.stdout)
    .context("Could not parse ytdlp::describe response")?;
  
  Ok(result)
}

pub async fn download(url: url::Url, filename: String, format_id: Option<String>) -> Result<()> {
  let mut cmd = Command::new("yt-dlp");
  cmd.arg("-o").arg(filename);
  if let Some(format_id) = format_id {
    cmd.arg("-f").arg(format_id);
  }
  cmd.arg(url.to_string());
  log::info!("ytdlp::download {:?}", &cmd);
  let output = cmd.output().await?;

  if !output.status.success() {
    // Err(output.stderr.to_string())
    log::error!("stdout: {:?}\nstderr: {:?}",
                std::str::from_utf8(&output.stdout).unwrap(),
                std::str::from_utf8(&output.stderr).unwrap());
    Err(Error::msg("Command download failed"))
  } else { Ok(()) }?;

  // let result : Video = serde_json::from_slice(&output.stdout)?;
  
  Ok(())
}
