use std::fs;
use anyhow::{Result, anyhow};

/// Find fileswith names starting with [name]
pub fn find_files_pat(dir: &String, name: &String) -> Result<Vec<String>> {
  let files = std::fs::read_dir(dir)?;
  let match_cb = |file : fs::DirEntry| {
    let cur_filename = file.file_name().into_string()
      .map_err(|e| anyhow!("filename not valid: {:?}", e))?;
    if cur_filename.starts_with(name) {
      let x = file.path().into_os_string().into_string()
        .map_err(|e| anyhow!("filename is not a valid utf8: {:?}", e))?;
      Ok(x)
    } else {
      // Error("not match")
      Err(anyhow!(""))
    }
  };
  let res = files.filter_map(
    |f_| f_.map_err(|e| anyhow!("file entry not valid: {:?}", e))
      .and_then(match_cb).ok())
    .collect();
  Ok(res)
}


pub fn find_file_pat(dir: &String, name: &String) -> Result<String> {
  let mut files = find_files_pat(dir, name)?;
  files.sort();
  if files.is_empty() {
    Err(anyhow!("Could not find downloaded file"))
  } else {
    Ok(files[0].clone())
  }
}
