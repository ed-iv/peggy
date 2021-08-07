use reqwest::Url;
use std::io::Write;
use std::fs::OpenOptions;
use rand::Rng;
use crate::*;

#[derive(Debug)]
pub struct Peggy {   
    pub url: String,
    pub last: String,
    pub contract: String,    
    pub limit: String,
    pub exclamations: [String; 4],
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
                println!(" ⏱️  Peggy is looking into the future...\n");
            },
            Err(err)  => {
                println!("Failed to update Peggy's timestamp: {}", err);
            }
        }
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
        let exclamation = self.get_exclamation();
        let message = match event_type.clone() {
            EventType::Bid => {          
                if let Some(from_account) = &event.from_account {
                    match &from_account.user.username {
                        Some(bidder) => {                                                                      
                            let mut message = format!(
                                "{exclamation}, {bidder} just bid {amount} {symbol} on {pegz_name}!", 
                                exclamation = exclamation,
                                bidder = bidder,                                
                                amount = in_eth(event.bid_amount.unwrap_or(Default::default()).as_str()),
                                symbol = symbol,      
                                pegz_name = pegz_name,                                                                  
                            );
                            if let Some(owner) = &event.asset.owner.user.username {
                                message = format!("{}\nWhat you goanna do about it, {}", message, owner);
                            }
                            message
                        },
                        None => {
                            format!(
                                "{}, somebody just bid {} {} on {}!", 
                                exclamation,                   
                                in_eth(event.bid_amount.unwrap_or(Default::default()).as_str()),
                                symbol,
                                pegz_name,                                                                
                            )
                        }
                    }
                } else {
                    format!(
                        "{}, somebody just bid {} {} on {}!", 
                        exclamation,                   
                        in_eth(event.bid_amount.unwrap_or(Default::default()).as_str()),
                        symbol,
                        pegz_name,                                                                
                    )
                }                             
            },
            EventType::List => {   
                match &event.asset.owner.user.username {
                    Some(owner) => {
                        format!(
                            "{exclamation}, {owner} just listed {pegz_name} for {price} {symbol}!",
                            exclamation = exclamation,
                            owner = owner,
                            pegz_name = pegz_name,
                            price = in_eth(event.starting_price.unwrap_or(Default::default()).as_str()),
                            symbol = symbol,
        
                        )
                    },
                    None => {
                        format!(
                            "{exclamation}, somebody just listed {pegz_name} for {price} {symbol}!",
                            exclamation = exclamation,
                            pegz_name = pegz_name,
                            price = in_eth(event.starting_price.unwrap_or(Default::default()).as_str()),
                            symbol = symbol,
        
                        )
                    }
                }
                
            },
            EventType::Sale => {
                match &event.asset.owner.user.username {
                    Some(new_owner) => {
                        format!(
                            "{exclamation}, {new_owner} just bought {pegz_name} for {amount} {symbol}!",
                            exclamation = exclamation,
                            new_owner = new_owner,
                            pegz_name = pegz_name,
                            amount = in_eth(event.total_price.unwrap_or(Default::default()).as_str()),
                            symbol = symbol,
                        )
                    },
                    None => {
                        format!(
                            "{exclamation}, some lucky bastard just bought {pegz_name} for {amount} {symbol}!",
                            exclamation = exclamation,                            
                            pegz_name = pegz_name,
                            amount = in_eth(event.total_price.unwrap_or(Default::default()).as_str()),
                            symbol = symbol,
                        )
                    },
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

