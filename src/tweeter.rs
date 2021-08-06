use serde::Deserialize;
use egg_mode::tweet::DraftTweet;
use egg_mode::Token;
use crate::*;

pub struct Tweeter {
    token: Token
}

impl Tweeter {
    pub fn new() -> Self {
        let con_key = dotenv::var("CONSUMER_KEY").unwrap();
        let con_secret = dotenv::var("CONSUMER_SECRET").unwrap();
        let con_token = egg_mode::KeyPair::new(con_key, con_secret);

        let access_token_key = dotenv::var("ACCESS_TOKEN_KEY").unwrap();
        let access_token_secret = dotenv::var("ACCESS_TOKEN_SECRET").unwrap();
        let access_token = egg_mode::KeyPair::new(access_token_key, access_token_secret);

        let token = egg_mode::Token::Access {consumer: con_token, access: access_token};

        Self { token }
    }

    pub async fn tweet(&self, notification: Notification) -> Result<(), Box<dyn std::error::Error>> {
                    
        let mut tweet = DraftTweet::new(notification.message);

        let handle = upload_media(
            &notification.image, 
            &media_types::image_gif(), 
            &self.token
        ).await?;
        tweet.add_media(handle.id);
        tweet.send(&self.token).await?;                

        Ok(())        
    }
}