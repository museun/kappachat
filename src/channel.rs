use uuid::Uuid;

use crate::{
    helix,
    store::{Image, ImageStore},
};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Channel {
    pub id: u64,

    pub display_name: String,

    pub login: String,
    pub description: String,

    pub image_id: Uuid,
    pub profile_image_url: String,

    pub show_timestamps: bool,
    pub show_user_list: bool,
    pub auto_join: bool,
}

impl Channel {
    pub fn new(user: helix::User) -> Self {
        let uuid = ImageStore::<Image>::get_id(&user.profile_image_url) //
            .unwrap_or_else(Uuid::new_v4);

        Self {
            id: user.id.parse().expect("valid id"),
            login: user.login,

            display_name: user.display_name,
            description: user.description,

            image_id: uuid,
            profile_image_url: user.profile_image_url,

            show_timestamps: true,
            show_user_list: true,
            auto_join: true,
        }
    }

    pub fn temporary(self) -> Self {
        Self {
            auto_join: false,
            ..self
        }
    }
}
