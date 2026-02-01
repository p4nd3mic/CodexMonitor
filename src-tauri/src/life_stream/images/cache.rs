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
    fn cache_path(&self, card_type: &str, entity_name: &str) -> PathBuf {
        let safe_name = entity_name.replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");

        self.root
            .join("Entities")
            .join(Self::type_to_folder(card_type))
            .join(&safe_name)
            .join("cover.jpg")
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
        let path = self.cache_path(card_type, entity_name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn save(&self, card_type: &str, entity_name: &str, data: &[u8]) -> PathBuf {
        let path = self.cache_path(card_type, entity_name);

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let _ = fs::write(&path, data);
        path
    }
}
