use reqwest::{Client, Url};
use serde::Deserialize;
use std::error::Error;

pub struct TmdbFetcher {
    client: Client,
    api_key: Option<String>,
}

#[derive(Deserialize)]
struct TmdbSearchResult {
    results: Vec<TmdbMedia>,
}

#[derive(Deserialize)]
struct TmdbMedia {
    poster_path: Option<String>,
}

impl TmdbFetcher {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn fetch(
        &self,
        title: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send + Sync>> {
        let api_key = match self.api_key.as_deref() {
            Some(key) => key,
            None => return Ok(None),
        };

        let mut search_url = Url::parse("https://api.themoviedb.org/3/search/multi")?;
        {
            let mut pairs = search_url.query_pairs_mut();
            pairs.append_pair("api_key", api_key);
            pairs.append_pair("query", title);
        }

        let search_result: TmdbSearchResult = self
            .client
            .get(search_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let poster_path = search_result
            .results
            .first()
            .and_then(|media| media.poster_path.as_ref());

        let poster_path = match poster_path {
            Some(path) => path,
            None => return Ok(None),
        };

        let image_url = format!("https://image.tmdb.org/t/p/w300{poster_path}");
        let image_bytes = self
            .client
            .get(image_url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();

        Ok(Some(image_bytes))
    }
}
