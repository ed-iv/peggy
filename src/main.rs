use serde::Deserialize;
// use reqwest::Error;
use egg_mode::error::{Error};
use egg_mode::tweet::DraftTweet;
use egg_mode::media::{upload_media, media_types};
mod types;
use core::time;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;


use reqwest::Url;

use crate::types::{Obj};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let mut last_update = SystemTime::now();
          
    loop {
        
        let mut timestamp = last_update
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();    

        println!("Looking for events after {}...", timestamp);
            
        let request_url = format!(
            "https://api.opensea.io/api/v1/events?asset_contract_address={contract}&only_opensea=false&offest={offset}&limit={limit}&occurred_after={occurred_after}",
            contract = "0x1eFf5ed809C994eE2f500F076cEF22Ef3fd9c25D",
            offset = 0,
            limit = 1,
            occurred_after = timestamp,
        );
        println!("{}", request_url);    
        let response: Obj = reqwest::get(&request_url).await?.json().await?;            
        
        if let Some(events) = response.asset_events {
            if (events.len() > 0) {
                for event in events {            
                    event.tweet().await?;                    
                }
                last_update = SystemTime::now();
            } else {
                println!("No events found...");    
            }
        }
        
        sleep(Duration::new(60, 0));   
    }
    
    Ok(())
}
