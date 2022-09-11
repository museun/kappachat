#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Channel {
    pub name: String,
    pub show_timestamps: bool,
    pub show_user_list: bool,
    pub auto_join: bool,
}

impl Channel {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
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
