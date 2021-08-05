use serde::Deserialize;
use egg_mode::tweet::DraftTweet;
use crate::*;

const WEI: u64 = 1_000_000_000_000_000_000;  
               
#[derive(Deserialize, Debug)]
pub struct Obj {
    pub asset_events: Option<Vec<Event>>,    
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

#[derive(Deserialize, Debug)]
pub struct Event {
    pub id: u128,    
    pub event_type: String, 
    pub asset: Pegz,    
    // pub payment_token: PaymentToken,
    // pub total_price: String,
    // pub bid_amount: Option<String>,
}

impl Event {    
    pub fn display(&self) {
        let output = format!(
            "Asset: PEGZ {} Event Type: {}",
            self.asset.id,
            self.event_type            
        );

        println!("{}", output);
    } 

    // pub fn sale_price_in_eth(&self) -> f64 {
    //     self.total_price.parse::<u128>().unwrap() as f64 / WEI as f64
    // }

    // pub fn bid_price_in_eth(&self) -> f64 {
    //     match &self.bid_amount {
    //         Some(bid) => bid.parse::<u128>().unwrap() as f64 / WEI as f64,
    //         None => 0.0,
    //     }
        
    // }
    
    pub async fn tweet(&self) -> Result<(), Box<dyn std::error::Error>> {
    
        let con_token = egg_mode::KeyPair::new(
            "Gfm6epmHQwKoGveDpxCYBx94K", 
            "hgcEDxLrLHR4NaZ3keW3KsolQCVOwok1sBcH3FDFKz8kZjsyyF"
        );

        let access_token = egg_mode::KeyPair::new(
            "1423130622197866502-FOkh5uMFgxSD5z4kMRHwNeisVhnJMr", 
            "H7lANRonTxXI54VKGqqty0UcSgIudzp3B09R8acM2roBW"
        );

        let token = egg_mode::Token::Access {
            consumer: con_token,
            access: access_token,
        };

        
        let text = format!(
            "Testing ... PEGZ #{} Event Type: {}",
            self.asset.name, 
            self.event_type
        );

        let image = reqwest::get(&self.asset.image_url)
            .await?
            .bytes()
            .await?;

        let mut tweet = DraftTweet::new(text.clone());

        let handle = upload_media(&image, &media_types::image_gif(), &token).await?;
        tweet.add_media(handle.id);
        tweet.send(&token).await?;        
        println!("Tweeting again: '{}'", text);

        Ok(())
    }
}
