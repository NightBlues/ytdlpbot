use anyhow::Result;

mod telegram_messages;
mod telegram;
mod ytdlp;
mod commands;

#[tokio::main]
async fn main() -> Result<()> {

  let telegram_token = std::env::var("TELEGRAM_TOKEN")
    .expect("Specify TELEGRAM_TOKEN env var.");
  let conf = telegram::Conf {
    token: telegram_token,
  };

  println!("Started...");
  let mut update_id : Option<i64> = None;
  loop {
    let res =
      telegram::get_updates(conf.clone(), update_id).await;
    let messages = match res {
      Ok((update_id_, messages)) => {
        // println!("update_id={:?}", update_id_);
        update_id = update_id_.map(|x| x + 1i64);
        messages
      },
      Err(e) => { println!("Error: {}", e); vec![] },
    };
    commands::react_messages(conf.clone(), messages.clone()).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await
  }

  // Ok(())
}
