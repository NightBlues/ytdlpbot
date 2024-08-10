use anyhow::Result;

mod config;
mod utils;
mod telegram_messages;
mod telegram;
mod ytdlp;
mod user_state;
mod ffmpeg;
mod format_chooser;
mod commands;

#[tokio::main]
async fn main() -> Result<()> {

  let telegram_token = std::env::var("TELEGRAM_TOKEN")
    .expect("Specify TELEGRAM_TOKEN env var.");
  let max_filesize : i64 = std::env::var("MAX_FILESIZE")
    .map_err(|x| x.to_string())
    .and_then(|x| x.parse::<i64>().map_err(|x| x.to_string()))
    .unwrap_or(50 * 1024 * 1024);
  let conf = config::Config {
    max_filesize,
    telegram_token,
    download_dir: "dl".to_string(),
  };
  if !std::fs::metadata(&conf.download_dir).unwrap().is_dir() {
    panic!("Download dir doesn not exist")
  }
  let state = user_state::State::new();
  // pretty_env_logger::init_timed();
  pretty_env_logger::formatted_timed_builder()
    .write_style(pretty_env_logger::env_logger::WriteStyle::Auto)
    .filter(Some("ytdlpbot"), log::LevelFilter::Debug)
    .filter(Some("reqwest"), log::LevelFilter::Info)
    .init();
  log::info!("Started...");
  let mut update_id : Option<i64> = None;
  let mut warm_up = true;
  loop {
    let res =
      telegram::get_updates(&conf.telegram_token, update_id).await;
    let messages = match res {
      Ok((update_id_, messages)) => {
        // log::debug!("update_id={:?}", update_id_);
        update_id = update_id_.map(|x| x + 1i64);
        messages
      },
      Err(e) => { log::error!("Error: {}", e); vec![] },
    };
    // ignore everything before start
    if warm_up {
      log::info!("Warmup to updateId = {:?}", update_id);
      warm_up = false;
      continue
    }
    commands::react_messages(&conf, &state, messages.clone()).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await
  }

  // Ok(())
}
