use serde::Deserialize;
use egg_mode::tweet::DraftTweet;
use crate::*;
use std::{collections::HashMap, str::FromStr};
use reqwest::Url;
use bytes::Bytes;
use std::io::Write;
use std::fs::{File, OpenOptions};

const WEI: u64 = 1_000_000_000_000_000_000;  
               
#[derive(Deserialize, Debug)]
pub struct Obj {
    pub asset_events: Vec<Event>,    
}

#[derive(Deserialize, Debug)]
pub struct Owner {
   pub user: Option<User>,
   pub profile_img_url: String,
   pub address: String, 
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub username: String,
}

#[derive(Deserialize, Debug)]
pub struct PaymentToken {
    pub symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct Pegz {
    pub id: u64,
    pub token_id: String,
    pub name: String,
    pub image_url: String,    
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum EventType {
    Bid,
    List,
    Sale,
    Unknown
}

impl From<&str> for EventType {
    fn from(input: &str) -> EventType {
        match input {
            "successful" => EventType::Sale,
            "bid_entered" => EventType::Bid,
            "created" => EventType::List,
            _ => EventType::Unknown,
        }
    }    
}

#[derive(Debug)]
pub struct Notification {    
    pub message: String,
    pub image: Bytes,
}

impl Notification {    
    pub fn new(event: Event) -> Result<Self, ()> {
        Err(())
    }
}

pub struct Peggy {   
    pub url: String,
    pub last: String,
    pub contract: String,    
    pub limit: String,
}

impl Peggy {
    pub fn new(
        url: String,
        last: String,
        contract: String,    
        limit: String,
    ) -> Self {        
        Self { 
            url,
            last,
            contract,
            limit,            
        }
    }

    pub fn update_last_fetch(&self) -> Result<(), Error> {
        let mut f = OpenOptions::new().write(true).truncate(true).open("../last-fetch")?;
        f.write_all(format!("{}", Utc::now().timestamp()).as_bytes())?;
        f.flush()?;
        Ok(())
    }

    pub async fn fetch_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        let url = self.build_url()?;
        println!("Fetching Events @ {}\n  »---> URL: {}", self.last, &url);
        let response: Obj = reqwest::get(url).await?.json().await?; 
        Ok(response.asset_events)
    }

    pub fn build_url(&self) -> Result<Url, Box<dyn std::error::Error>> {
        let params = vec![
            ("asset_contract_address", &self.contract),
            ("occurred_after", &self.last),
            ("limit", &self.limit)
        ];
        let url = Url::parse_with_params(self.url.as_str(), &params)?;
        Ok(url)
    }


    pub async fn get_notification(&self, event: Event) -> Result<Notification, Box<dyn std::error::Error>> {
        let pegz_name = format!("PEGZ #{}", &event.asset.token_id);   
        let mut symbol = "";
        if let Some(payment_token) = &event.payment_token {
            symbol = payment_token.symbol.as_str();
        } 
        let image = event.get_image().await?;

        let event_type: EventType = event.event_type.as_str().into();
        let message = match event_type.clone() {
            EventType::Bid => {                                
                format!(
                    "{} has a bid of {} {}!",                    
                    pegz_name,
                    in_eth(event.bid_amount.unwrap_or(Default::default()).as_str()),
                    symbol,                                        
                )
            },
            EventType::List => {                
                format!(
                    "{} has just been listed for {} {}!",
                    pegz_name,
                    in_eth(event.starting_price.unwrap_or(Default::default()).as_str()),
                    symbol,

                )
            },
            EventType::Sale => {
                
                format!(
                    "{} just sold for {} {}!",
                    pegz_name,
                    in_eth(event.total_price.unwrap_or(Default::default()).as_str()),
                    symbol,
                )
            },
            EventType::Unknown => {
                format!("Unknown event type")
            },
        };

    
        let notification = Notification {            
            message,
            image
        };

        Ok(notification)
    }
}

#[derive(Deserialize, Debug)]
pub struct Event {
    pub id: u128,    
    pub event_type: String, 
    pub asset: Pegz,    
    pub payment_token: Option<PaymentToken>,
    pub total_price: Option<String>,
    pub bid_amount: Option<String>,
    pub starting_price: Option<String>,
}

pub fn in_eth(amount: &str) -> f64 {
    amount.parse::<u128>().unwrap() as f64 / WEI as f64
}


impl Event {    
    pub async fn get_image(&self) -> Result<Bytes, Box<dyn std::error::Error>> {
        let image: Bytes = reqwest::get(&self.asset.image_url)
            .await?
            .bytes()
            .await?;

        Ok(image)
    }

}
