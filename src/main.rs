use serde::Deserialize;
// use reqwest::Error;
use egg_mode::error::{Error};
use egg_mode::tweet::DraftTweet;
use egg_mode::media::{upload_media, media_types};
mod types;
use core::time;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use dotenv::dotenv;
use std::env;
use types::*;
use chrono::prelude::*;
use chrono::{DateTime, Utc};
mod tweeter;

use tweeter::Tweeter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        
    let timestamp = include_str!("../last-fetch");
    
    let mut peggy = Peggy::new(
        String::from("https://api.opensea.io/api/v1/events"),
        String::from(timestamp.trim()),
        String::from("0x1eFf5ed809C994eE2f500F076cEF22Ef3fd9c25D"),                        
        format!("{}", 20),
    );
    let tweeter = Tweeter::new();

    loop {        
        if let Ok(events) = peggy.fetch_events().await {
            if events.len() > 0 {
                for event in events.into_iter().rev() {
                    let mut event_id = &event.id.clone();
                    if let EventType::Unknown = &event.event_type.as_str().into() {
                        println!("  »---> Skipping unknown EventType \n");
                        peggy.update_last_fetch().unwrap_or(println!("Failed to update Peggy's last-fetch timestamp"));
                        continue;                    
                    }
                    if let Ok(notification) = peggy.get_notification(event).await {
                        println!("  »---> {}\n", notification.message);                
                        if let Ok(tweet) = tweeter.tweet(notification).await {
                            // Do nothing
                        } else {
                            println!("  »---> Failed to tweet notification\n");
                        }
                        peggy.update_last_fetch().unwrap_or(println!("Failed to update Peggy's last-fetch timestamp"));
                    } else {
                        println!("  »---> Unable to get notification from Event with ID: {}\n", event_id);
                    }                                                     
                }
            } else {
                println!("  »---> No events found...Sleeping\n");
            }
        } else {
            println!("  »---> Unable to fetch events...trying again in 1 min\n ");
        }
        sleep(Duration::new(60, 0));   
    }
    Ok(())
}
