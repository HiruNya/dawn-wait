use dawn::{gateway::shard::{Config, Shard, Event}, http::Client};
use dawn_wait::Listener;
use futures::StreamExt;
use tokio::time::timeout;

use std::{env::var, time::Duration};

#[tokio::main]
async fn main() {
    // Boring Boilerplate stuff:
    dotenv::dotenv().expect("Could not load .env file");
    let token = var("DISCORD_TOKEN").expect("Could not find DISCORD_TOKEN environmental variable");
    let client_ = Client::new(&token);
    let mut config = Config::builder(token);
    config.guild_subscriptions(false)
        .http_client(client_.clone());
    let config = config.build();
    let gateway = Shard::new(config).await.expect("Error connecting to shard");
    let listener_ = Listener::default(); // Creating a `Listener`.
    // `Listener::handle` takes in a Stream of Events and returns a Stream of Events.
    let mut events = listener_.handle(gateway.events().await);
    while let Some(event) = events.next().await {
        let client = client_.clone();
        let listener = listener_.clone();
        tokio::spawn(async move {
            if let Event::MessageCreate(message) = event {
                if message.content.starts_with("!example") {
                    if let Err(e) = client.create_message(message.channel_id)
                        .content("Say Hello!")
                        .await {
                        println!("An error occurred when sending a message!\n  {:?}", e)
                    } else {
                        // This waits for 1 message which contains the word "hello".
                        // However this will wait forever so we introduce a timeout of 10 seconds.
                        if let Ok(_msg) = timeout(Duration::from_secs(10), listener.wait_for(message.channel_id, |msg| msg.content.contains("hello"))).await {
                            client.create_message(message.channel_id)
                                .content("Thanks for saying hello!")
                                .await
                                .unwrap_or_else(|e| panic!("An error occurred when sending a message:\n {:?}", e));
                        } else {
                            client.create_message(message.channel_id)
                                .content("No one said hello :(")
                                .await
                                .unwrap_or_else(|e| panic!("An error occurred when sending a message:\n {:?}", e));
                        }
                    }
                }
            }
        });
    }
}
