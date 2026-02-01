mod cache;
mod fetchers;

pub use cache::ImageCache;
pub use fetchers::TmdbFetcher;

use std::path::PathBuf;

use crate::life_stream::types::{CardImage, ImageStatus};

pub struct ImageService {
    cache: ImageCache,
    tmdb: TmdbFetcher,
}

impl ImageService {
    pub fn new(cache_root: PathBuf, tmdb_api_key: Option<String>) -> Self {
        Self {
            cache: ImageCache::new(cache_root),
            tmdb: TmdbFetcher::new(tmdb_api_key),
        }
    }

    /// Fetch image for a card, checking cache first.
    pub async fn fetch_image(&self, card_type: &str, entity_name: &str) -> CardImage {
        if let Some(cached) = self.cache.get(card_type, entity_name) {
            return CardImage {
                url: Some(cached.to_string_lossy().to_string()),
                status: ImageStatus::Ready,
                source: Some("cache".to_string()),
            };
        }

        let result = match card_type {
            "media" => self.tmdb.fetch(entity_name).await,
            "meal" | "food" => Ok(None),
            _ => Ok(None),
        };

        match result {
            Ok(Some(image_data)) => {
                match self.cache.save(card_type, entity_name, &image_data) {
                    Ok(path) => CardImage {
                        url: Some(path.to_string_lossy().to_string()),
                        status: ImageStatus::Ready,
                        source: Some("fetched".to_string()),
                    },
                    Err(err) => {
                        eprintln!(
                            "Failed to cache image for {} ({}): {}",
                            card_type, entity_name, err
                        );
                        CardImage {
                            url: None,
                            status: ImageStatus::Missing,
                            source: Some(format!("cache error: {err}")),
                        }
                    }
                }
            }
            Ok(None) => CardImage {
                url: None,
                status: ImageStatus::UploadPrompt,
                source: None,
            },
            Err(err) => CardImage {
                url: None,
                status: ImageStatus::Missing,
                source: Some(format!("error: {err}")),
            },
        }
    }
}
