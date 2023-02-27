// example
use mastodon_async::entities::notification::NotificationType;
use mastodon_async::helpers::toml; // requires `features = ["toml"]`
use mastodon_async::prelude::*;
use mastodon_async::{helpers::cli, Result};
use std::env;

use futures_util::TryStreamExt;
// use log::{as_serde, info};

#[tokio::main] // requires `features = ["mt"]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mastodata: &String = &args[1];

    let mut count = 0u32;
    loop {
        run(mastodata).await?;
        count += 1;
        if count == 10 {
            println!("Retried {count} times, I had enough!");
            break;
        }
    }
    Ok(())
}

async fn run(mastodata: &String) -> Result<()> {
    let mastodon = if let Ok(data) = toml::from_file(mastodata) {
        Mastodon::from(data)
    } else {
        register(mastodata).await?
    };
    let stream = mastodon.stream_notifications().await?;
    println!(
        "watching mastodon for notifications. This will run forever, press Ctrl+C to kill the program."
    );
    stream
        .try_for_each(|(event, mastodon)| async move {
            if let Event::Notification(notif) = event {
                notif_handler(notif, mastodon).await?;
            }
            Ok(())
        })
        .await?;
    Ok(())
}

async fn register(mastodata: &String) -> Result<Mastodon> {
    let registration = Registration::new("https://ecoevo.social")
        .client_name("mastodon-async-examples")
        .build()
        .await?;
    let mastodon = cli::authenticate(registration).await?;

    // Save app data for using on the next run.
    toml::to_file(&mastodon.data, mastodata)?;

    Ok(mastodon)
}

async fn notif_handler(notif: Notification, mastodon: Mastodon) -> Result<()> {
    if notif.notification_type == NotificationType::Mention && from_ecoevo(&notif) {
        mastodon.reblog(&notif.status.clone().unwrap().id).await?;
        println!("Rebloged {}", notif.status.unwrap().url.unwrap());
    }
    Ok(())
}

fn from_ecoevo(notif: &Notification) -> bool {
    notif.account.url.contains("https://ecoevo.social/")
}
