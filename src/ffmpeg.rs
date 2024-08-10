use std::path;
use anyhow::{Result, Error, anyhow};
use tokio::process::Command;
use crate::user_state::{CutInterval};


/// invent output file name
fn out_file(filepath: &String) -> Result<String> {
  log::debug!("ffmpeg out_file({})", filepath);
  let filepath = path::Path::new(filepath);
  let filename = filepath.file_name()
    .ok_or(anyhow!("file name path part is empty"))?;
  let parent = filepath.parent();
  let filename = path::Path::new(filename);
  let ext = filename.extension()
    .ok_or(anyhow!("file extension is empty"))?
    .to_str()
    .ok_or(anyhow!("file extenstion contains non utf8 characters"))?;
  let file_stem = filename.file_stem()
    .ok_or(anyhow!("file name is empty"))?
    .to_str()
    .ok_or(anyhow!("file extenstion contains non utf8 characters"))?;
  let newfilename = format!("{}.cut.{}", file_stem, ext);
  if let Some(parent) = parent {
    let path = parent.join(newfilename);
    let result = path.to_str()
      .ok_or(anyhow!("new file path contains non utf8 characters"))?;
    Ok(result.to_string())
  } else {
    Ok(newfilename)
  }
}


/// Run ffmpeg to cut part of video.
pub async fn cut(filename: &String, cut_interval: CutInterval) -> Result<String> {
  let outfile = out_file(filename)?;
  let mut cmd = Command::new("ffmpeg");
  cmd.arg("-i").arg(filename)
    .arg("-ss").arg(cut_interval.start.to_string())
    .arg("-to").arg(cut_interval.end.to_string())
    .arg("-c:v").arg("copy")
    .arg("-c:a").arg("copy")
    .arg(&outfile);
  log::info!("ffmpeg::cut {:?}", &cmd);
  let output = cmd.output().await?;

  if !output.status.success() {
    // Err(output.stderr.to_string())
    log::error!("stdout: {:?}\nstderr: {:?}",
                std::str::from_utf8(&output.stdout).unwrap(),
                std::str::from_utf8(&output.stderr).unwrap());
    Err(Error::msg("Command ffmpeg::cut failed"))
  } else { Ok(()) }?;

  // let result : Video = serde_json::from_slice(&output.stdout)?;
  
  Ok(outfile)
}
