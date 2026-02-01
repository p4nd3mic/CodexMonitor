use std::error::Error;

#[derive(Default)]
pub struct FoodImageFetcher;

impl FoodImageFetcher {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        _name: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send + Sync>> {
        Ok(None)
    }
}
