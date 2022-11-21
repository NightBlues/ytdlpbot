use serde::{Deserialize, Serialize};
use anyhow::{Result, Error, Context, anyhow};
use tokio::process::Command;



#[derive(Deserialize, Serialize, Debug)]
pub struct Format {
  pub format_id: String,
  #[serde(default)]
  pub filesize_approx: Option<i64>,
  pub vcodec: String,
  pub acodec: String,
  pub ext: String,
  pub height: Option<i64>,
  pub width: Option<i64>,
  #[serde(default)]
  pub filesize: Option<i64>,
}

impl Format {
  pub fn get_filesize(&self) -> Option<i64> {
    self.filesize.or(self.filesize_approx)
  }
}



#[derive(Deserialize, Serialize, Debug)]
pub struct Video {
  pub id: String,
  pub title: String,
  pub filename: String,
  pub ext: String,
  #[serde(default)]
  pub fps: f64,
  pub width: i64,
  pub height: i64,
  pub vcodec: String,
  pub acodec: String,
  pub vbr: f64,
  pub format: String,
  pub format_id: String,
  pub formats: Vec<Format>,
  #[serde(default, alias="filesize")]
  pub filesize_approx: Option<i64>,
  pub duration: i64,
}

impl std::fmt::Display for Video {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Video: {}.{} {} {:?}bytes {}secs",
           self.id, self.ext, self.title, self.filesize_approx, self.duration)
  }
}


pub async fn describe(url: url::Url) -> Result<Video> {
  let output = Command::new("yt-dlp").arg("-j").arg(url.to_string()).output();
  let output = output.await?;

  if !output.status.success() {
    // Err(output.stderr.to_string())
    println!("stdout: {:?}\nstderr: {:?}",
             std::str::from_utf8(&output.stdout).unwrap(),
             std::str::from_utf8(&output.stderr).unwrap());
    Err(Error::msg("Command failed"))
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
  let output = cmd.output().await?;

  if !output.status.success() {
    // Err(output.stderr.to_string())
    println!("stdout: {:?}\nstderr: {:?}",
             std::str::from_utf8(&output.stdout).unwrap(),
             std::str::from_utf8(&output.stderr).unwrap());
    Err(Error::msg("Command failed"))
  } else { Ok(()) }?;

  // let result : Video = serde_json::from_slice(&output.stdout)?;
    
  Ok(())
}
