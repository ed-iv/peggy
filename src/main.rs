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

use std::fmt::Display;


use reqwest::Url;

use crate::types::{Obj};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        
    let timestamp = Utc::now().timestamp();
    let mut peggy = Peggy::new(
        String::from("https://api.opensea.io/api/v1/events"),
        format!("{}", "1628276093"),
        String::from("0x1eFf5ed809C994eE2f500F076cEF22Ef3fd9c25D"),                        
        format!("{}", 20),
    );

    let tweeter = Tweeter::new();

    
    loop {        
        let events = peggy.fetch_events().await?;
        if events.len() > 0 {
            for event in events.into_iter().rev() {
                if let EventType::Unknown = &event.event_type.as_str().into() {
                    continue;
                }
                let notification = peggy.get_notification(event).await?;                
                println!("{}", notification.message);
                tweeter.tweet(notification).await?;
                peggy.last = format!("{}", Utc::now().timestamp());

            }
        } else {
            println!("No events found...Sleeping");
        }
        sleep(Duration::new(60, 0));   
    }
    
    Ok(())
}
