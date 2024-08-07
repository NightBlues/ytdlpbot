#[derive(Clone)]
pub struct Config {
  pub max_filesize: i64,
  // pub vcodec_exclude: Vec<String>,
  pub telegram_token: String,
  pub download_dir: String,
}
