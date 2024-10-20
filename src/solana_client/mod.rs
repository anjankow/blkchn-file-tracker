use crate::error::Error;
use crate::event::Event;

pub struct SolanaClient {
    url: String,
}

impl SolanaClient {
    pub fn new(url: &str) -> SolanaClient {
        SolanaClient {
            url: url.to_string(),
        }
    }

    pub fn process_events(&self, rx: std::sync::mpsc::Receiver<Event>) -> Result<(), Error> {
        for event in rx {
            println!("Consumer received an event: {}", event);
        }
        Ok(())
    }
}
