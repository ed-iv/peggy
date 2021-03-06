use reqwest::Url;
use std::io::Write;
use std::fs::OpenOptions;
use rand::Rng;
use num_format::{Buffer, Locale, ToFormattedString};
use crate::*;


const WEI: u64 = 1_000_000_000_000_000_000;  
const MIN_BID: f64 = 50.0; 

#[derive(Debug)]
pub struct Peggy {   
    pub url: String,
    pub last: String,
    pub contract: String,    
    pub limit: String,
    pub exclamations: [String; 4],
}

pub fn in_eth(amount: &str) -> f64 {
    amount.parse::<u128>().unwrap() as f64 / WEI as f64
}

pub fn in_stable_coin(amount: &str, symbol: &str) -> f64 {
    match symbol { 
        "USDC" => {
            amount.parse::<u128>().unwrap() as f64 / 1_000_000 as f64   
        },
        "DAI" => {
            amount.parse::<u128>().unwrap() as f64 /                  1_000_000_000_000_000_000.0
        },
        _ => 0.0
    }
}

pub fn format_num(num: f64) -> String {
    if (num < MIN_BID) {
        return String::default();
    }
    let whole: u32 = num.floor() as u32;
    let part = num - num.floor(); 
    let mut partString = &format!("{:.2}", part)[2..];
    partString = partString.trim_end_matches(|c| c == '0');

    let whole = whole.to_formatted_string(&Locale::en);

    if (part > 0.0) {
        format!("{}.{}", whole, partString)
    } else {
        format!("{}", whole)
    }
}

pub fn format_currency(amount: &str, symbol: &str) -> String {
   match symbol {
        "WETH" => format_num(in_eth(amount)),   
        "ETH" => format_num(in_eth(amount)),
        // "USDC" => format_num(in_stable_coin(amount, symbol)),
        // "DAI" => format_num(in_stable_coin(amount, symbol)),
        _ => String::default(),
   }
    
}

impl Peggy {
    pub fn new(
        url: String,
        last: String,
        contract: String,    
        limit: String,
    ) -> Self {
        let exclamations = [
            String::from("Holy shit"),
            String::from("Good god"),
            String::from("LFG!"),
            String::from("Oh heavens"),
        ];

        Self { 
            url,
            last,
            contract,
            limit,   
            exclamations,         
        }
    }

    pub fn get_exclamation(&self) -> String {        
        let len = self.exclamations.len();
        let mut rng = rand::thread_rng();
        let index: usize = rng.gen::<usize>() % len;
        self.exclamations[index].clone()       
    }

    fn update_fetch_file (&self, timestamp: &String) -> Result<(), Error> {
        let mut f = OpenOptions::new().write(true).truncate(true).open("./last-fetch")?;        
        f.write_all(timestamp.as_bytes())?;
        f.flush()?;
        Ok(())
    }

    pub fn update_last_fetch(&mut self) {
        let now = format!("{}", Utc::now().timestamp());
        match self.update_fetch_file(&now) {
            Ok(_) => {
                self.last = now;
                println!(" ??????  Peggy is looking into the future...\n");
            },
            Err(err)  => {
                println!("Failed to update Peggy's timestamp: {}", err);
            }
        }
    }

    pub fn start_from_last_fetch(&mut self) {        
        self.last = String::from(include_str!("../last-fetch"));
    }

    pub async fn fetch_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        let url = self.build_url()?;        
        println!("Fetching Events @ {}\n", self.last);
        let response: Obj = reqwest::get(url).await?.json().await?; 
        let events= response.asset_events
            .into_iter()
            .rev()
            .filter(|e| {
                EventType::from(e.event_type.as_str()) != EventType::Unknown 
            })
            .collect();       
        Ok(events)
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
                let amount = format_currency(
                    event.bid_amount.unwrap_or(Default::default()).as_str(), 
                    symbol
                );
                if let Some(from_account) = &event.from_account {
                    if let Some(user) = &from_account.user {
                        match &user.username {
                            Some(bidder) => {                                                                      
                                let mut message = format!(
                                    "{bidder} just bid {amount} {symbol} on {pegz_name}!",                                 
                                    bidder = bidder,                                
                                    amount = amount,
                                    symbol = symbol,      
                                    pegz_name = pegz_name,                                                                  
                                );                                
                                message
                            },
                            None => {
                                format!(
                                    "Somebody just bid {} {} on {}!",                                                    
                                    amount,
                                    symbol,
                                    pegz_name,                                                                
                                )
                            }
                        }
                    } else {
                        format!(
                            "Somebody just bid {} {} on {}!",                                                    
                            amount,
                            symbol,
                            pegz_name,                                                                
                        )
                    }
                    
                } else {
                    format!(
                        "Somebody just bid {} {} on {}!",                         
                        amount,
                        symbol,
                        pegz_name,                                                                
                    )
                }                             
            },
            EventType::List => {   
                let auction_type = &event.auction_type.ok_or("No auction type.").unwrap();
                let mut owner = String::from("someone");
                if let Some(user) = &event.asset.owner.user {
                    if let Some(name) =  &user.username {
                        owner = name.clone(); 
                    }
                }
                let starting_price = format_currency(
                    event.starting_price.unwrap_or(Default::default()).as_str(), 
                    symbol
                );
                match auction_type.as_str() {
                    "english" => {
                        format!(
                            "{owner} just started an auction for {pegz_name} with a starting price of {price} {symbol}!",                            
                            owner = owner,
                            pegz_name = pegz_name,
                            price = starting_price,
                            symbol = symbol,
                        )
                    },                     
                    _ => {
                        format!(
                            "{owner} just listed {pegz_name} for {price} {symbol}!",                            
                            owner = owner,
                            pegz_name = pegz_name,
                            price = starting_price,
                            symbol = symbol,
                        )
                    }
                }
            },
            EventType::Sale => {
                let total_price = format_currency(
                    event.total_price.unwrap_or(Default::default()).as_str(), 
                    symbol
                );
                if let Some(user) = &event.asset.owner.user {
                    match &user.username {
                        Some(new_owner) => {
                            format!(
                                "{new_owner} just bought {pegz_name} for {amount} {symbol}!",                            
                                new_owner = new_owner,
                                pegz_name = pegz_name,
                                amount = total_price,
                                symbol = symbol,
                            )
                        },
                        None => {
                            format!(
                                "Some lucky bastard just bought {pegz_name} for {amount} {symbol}!",                            
                                pegz_name = pegz_name,
                                amount = total_price,
                                symbol = symbol,
                            )
                        },
                    }  
                } else {
                    format!(
                        "Some lucky bastard just bought {pegz_name} for {amount} {symbol}!",                            
                        pegz_name = pegz_name,
                        amount = total_price,
                        symbol = symbol,
                    )
                }
                        
            },
            EventType::Offer => {    
                let amount = format_currency(
                    event.bid_amount.unwrap_or(Default::default()).as_str(), 
                    symbol
                );      
                if (amount.is_empty()) {
                    String::default()
                } else {
                    if let Some(from_account) = &event.from_account {
                        if let Some(user) = &from_account.user {
                            match &user.username {
                                Some(bidder) => {                                                                      
                                    let message = format!(
                                        "{bidder} just offered {amount} {symbol} for {pegz_name}!",                                 
                                        bidder = bidder,                                
                                        amount = amount,                                
                                        symbol = symbol,      
                                        pegz_name = pegz_name,                                                                  
                                    );                                
                                    message
                                },
                                None => {
                                    format!(
                                        "Somebody just offered {} {} for {}!",                                                    
                                        amount,
                                        symbol,
                                        pegz_name,                                                                
                                    )
                                }
                            }
                        } else {
                            format!(
                                "Somebody just offered {} {} for {}!",                                                    
                                amount,
                                symbol,
                                pegz_name,                                                                
                            )
                        }
                        
                    } else {
                        format!(
                            "Somebody just offered {} {} for {}!",                         
                            amount,
                            symbol,
                            pegz_name,                                                                
                        )
                    } 
                }
                                            
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

