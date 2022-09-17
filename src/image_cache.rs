use std::collections::HashMap;

use egui_extras::RetainedImage;
use uuid::Uuid;

#[derive(Default)]
pub struct ImageCache {
    pub map: HashMap<Uuid, RetainedImage>,
}

impl ImageCache {
    pub fn get_id(&self, uuid: Uuid) -> Option<&RetainedImage> {
        self.map.get(&uuid)
    }

    pub fn has_id(&self, id: Uuid) -> bool {
        self.map.contains_key(&id)
    }

    pub fn add(&mut self, id: Uuid, image: RetainedImage) {
        log::debug!("image cache: adding: {id}");
        self.map.insert(id, image);
    }
}
