use anyhow::Result;

mod telegram_messages;
mod telegram;
mod ytdlp;
mod commands;

#[tokio::main]
async fn main() -> Result<()> {

  let telegram_token = std::env::var("TELEGRAM_TOKEN")
    .expect("Specify TELEGRAM_TOKEN env var.");
  let vcodec_exclude = std::env::var("VCODEC_EXCLUDE")
    .unwrap_or("".to_string());
    // .expect("Specify VCODEC_EXCLUDE env var.");
  let vcodec_exclude : Vec<String> = vcodec_exclude.split(",")
    .into_iter()
    .map(|x| x.to_string()).collect();
  let max_filesize : i64 = std::env::var("MAX_FILESIZE")
    .map_err(|x| x.to_string())
    .and_then(|x| x.parse::<i64>().map_err(|x| x.to_string()))
    .unwrap_or(50 * 1024 * 1024);
  let conf = commands::Config {
    max_filesize,
    vcodec_exclude,
    telegram_token,
  };
  let state = commands::State::new();
  println!("Started...");
  let mut update_id : Option<i64> = None;
  loop {
    let res =
      telegram::get_updates(&conf.telegram_token, update_id).await;
    let messages = match res {
      Ok((update_id_, messages)) => {
        // println!("update_id={:?}", update_id_);
        update_id = update_id_.map(|x| x + 1i64);
        messages
      },
      Err(e) => { println!("Error: {}", e); vec![] },
    };
    commands::react_messages(&conf, &state, messages.clone()).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await
  }

  // Ok(())
}
