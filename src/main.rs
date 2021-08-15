mod events;
mod tweeter;
mod peggy;

use egg_mode::error::{Error};
use egg_mode::media::{upload_media, media_types};
use std::time::Duration;
use std::thread::sleep;
use chrono::Utc;

use events::*;
use peggy::Peggy;



use tweeter::Tweeter;

#[tokio::main]
async fn main(){
        
    let timestamp = include_str!("../last-fetch");
    
    let mut peggy = Peggy::new(
        String::from("https://api.opensea.io/api/v1/events"),
        String::from(timestamp.trim()),
        String::from("0x1eFf5ed809C994eE2f500F076cEF22Ef3fd9c25D"),                        
        format!("{}", 20),
    );
    peggy.start_from_last_fetch();
    let tweeter = Tweeter::new();

    loop {                
        match peggy.fetch_events().await {
            Ok(events)  => {
                if !events.is_empty() {
                    for event in events {
                        let mut event_id = &event.id.clone();                    
                        if let Ok(notification) = peggy.get_notification(event).await {
                            println!(" 🎯 {}", notification.message);                
                            // if let Err(err) = tweeter.tweet(notification).await {
                            //     println!("  💀 Failed to tweet notification: {}", err);
                            // } else {
                            //     println!(" 🦤  Peggy tweeted notification");
                            // }                                                                           
                        } else {
                            println!("  💀 Unable to get notification from Event with ID: {}", event_id);
                        }                                                     
                    }
                    peggy.update_last_fetch();
                }else {
                    println!(" 😔 Peggy is sad; no events found");
                }                    
            },
            Err(err) => {
                println!(" 💀 Unable to fetch events: {}", err);
            }
        }        
        println!("\n 💤 Peggy is Sleeping 💤 \n");        
        sleep(Duration::new(60, 0));   
    }
}
