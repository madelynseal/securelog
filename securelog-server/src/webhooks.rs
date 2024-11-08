use crate::sql;
use crate::sql::webhooks::Webhook;

pub async fn send_message(message: &str) -> sql::Result<()> {
    info!("sending webhook message {} ", message);
    for webhook in sql::webhooks::get_webhooks().await? {
        send_message_inner(message, &webhook).await;
    }

    Ok(())
}

pub async fn send_message_inner(message: &str, wh: &Webhook) {
    use webhook::client::WebhookClient;

    let client = WebhookClient::new(&wh.get_url());

    match client
        .send(|msg| msg.content(message).username(&wh.username))
        .await
    {
        Ok(_) => {}
        Err(e) => {
            warn!("Error running webhook {}: {}", wh.name, e);
        }
    }
}
