use std::fs;
use std::path::PathBuf;

pub struct ImageCache {
    root: PathBuf,
}

impl ImageCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Cache path: {root}/Entities/{type}/{name}/cover.jpg
    fn cache_path(&self, card_type: &str, entity_name: &str) -> Result<PathBuf, String> {
        let safe_name = entity_name
            .replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            .replace("..", "_");

        let path = self
            .root
            .join("Entities")
            .join(Self::type_to_folder(card_type))
            .join(&safe_name)
            .join("cover.jpg");

        if path.components().any(|component| component == std::path::Component::ParentDir) {
            return Err("Path traversal attempt detected".to_string());
        }

        Ok(path)
    }

    fn type_to_folder(card_type: &str) -> &str {
        match card_type {
            "media" => "Media",
            "meal" | "food" => "Food",
            "delivery" => "Delivery",
            _ => "Misc",
        }
    }

    pub fn get(&self, card_type: &str, entity_name: &str) -> Option<PathBuf> {
        let path = self.cache_path(card_type, entity_name).ok()?;
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn save(
        &self,
        card_type: &str,
        entity_name: &str,
        data: &[u8],
    ) -> Result<PathBuf, std::io::Error> {
        let path = self.cache_path(card_type, entity_name).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, err)
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, data)?;
        Ok(path)
    }
}
