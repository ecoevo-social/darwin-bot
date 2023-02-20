// example

use mastodon_async::entities::notification::NotificationType;
use mastodon_async::helpers::toml; // requires `features = ["toml"]`
use mastodon_async::prelude::*;
use mastodon_async::{helpers::cli, Result};

use futures_util::TryStreamExt;
// use log::{as_serde, info};

#[tokio::main] // requires `features = ["mt"]
async fn main() -> Result<()> {
    run().await?;

    Ok(())
}

async fn run() -> Result<()> {
    let mastodon = if let Ok(data) = toml::from_file("mastodon-data.toml") {
        Mastodon::from(data)
    } else {
        register().await?
    };
    let stream = mastodon.stream_user().await?;
    println!(
        "watching mastodon for events. This will run forever, press Ctrl+C to kill the program."
    );
    stream
        .try_for_each(|(event, mastodon)| async move {
            match event {
                Event::Notification(notif) => notif_handler(notif, mastodon).await?,
                _ => (),
            }
            Ok(())
        })
        .await?;
    Ok(())
}

async fn register() -> Result<Mastodon> {
    let registration = Registration::new("https://ecoevo.social")
        .client_name("mastodon-async-examples")
        .build()
        .await?;
    let mastodon = cli::authenticate(registration).await?;

    // Save app data for using on the next run.
    toml::to_file(&mastodon.data, "mastodon-data.toml")?;

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
