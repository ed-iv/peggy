use serde::Deserialize;
use bytes::Bytes;
               
#[derive(Deserialize, Debug)]
pub struct Obj {
    pub asset_events: Vec<Event>,    
}

#[derive(Deserialize, Debug)]
pub struct Person {
   pub user: Option<User>,      
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub username: Option<String>,
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
    pub owner: Person, 
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Bid,
    List,
    Offer,
    Sale,
    Unknown
}

impl From<&str> for EventType {
    fn from(input: &str) -> EventType {
        match input {
            "successful" => EventType::Sale,
            "bid_entered" => EventType::Bid,
            "created" => EventType::List,
            "offer_entered" => EventType::Offer,
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

#[derive(Deserialize, Debug)]
pub struct Event {
    pub id: u128,    
    pub event_type: String, 
    pub auction_type: Option<String>, 
    pub asset: Pegz,    
    pub payment_token: Option<PaymentToken>,
    pub total_price: Option<String>,
    pub bid_amount: Option<String>,
    pub starting_price: Option<String>,
    pub seller: Option<Person>,
    pub from_account: Option<Person>,
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
