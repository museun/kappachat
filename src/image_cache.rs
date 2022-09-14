use std::collections::HashMap;

use egui_extras::RetainedImage;
use uuid::Uuid;

#[derive(Default)]
pub struct ImageCache {
    pub map: HashMap<Uuid, RetainedImage>,
    pub lookup: HashMap<String, Uuid>,
}

impl ImageCache {
    pub fn get(&self, key: &str) -> Option<&RetainedImage> {
        self.map.get(self.lookup.get(key)?)
    }

    pub fn has(&self, key: &str) -> bool {
        self.lookup.contains_key(key)
    }

    pub fn has_id(&self, id: Uuid) -> bool {
        self.map.contains_key(&id)
    }

    pub fn add(&mut self, name: impl ToString, id: Uuid, image: RetainedImage) {
        self.map.insert(id, image);
        self.lookup.insert(name.to_string(), id);
    }
}
