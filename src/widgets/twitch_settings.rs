use std::{borrow::Cow, collections::HashMap};

use egui::{Align, Grid, Label, Layout, RichText, TextEdit};

#[derive(Default)]
pub struct TwitchSettingsState {
    show_password: HashMap<u64, bool>,
}

use crate::{state::State, EnvConfig};

pub struct TwitchSettings<'a> {
    config: &'a mut EnvConfig,
    state: &'a mut TwitchSettingsState,
}

impl<'a> TwitchSettings<'a> {
    pub fn new(config: &'a mut EnvConfig, state: &'a mut TwitchSettingsState) -> Self {
        Self { config, state }
    }

    fn id() -> &'static str {
        std::any::type_name::<Self>()
    }

    pub fn display(self, ui: &mut egui::Ui) {
        Grid::new(Self::id())
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for ((label, validator, label_maker), (value, password)) in
                    Self::LABELS.into_iter().zip([
                        (&mut self.config.twitch_name, false),
                        (&mut self.config.twitch_oauth_token, true),
                        (&mut self.config.twitch_client_id, false),
                        (&mut self.config.twitch_client_secret, true),
                    ])
                {
                    match validator(&value).display() {
                        Some(requirements) => {
                            ui.add(Label::new(
                                RichText::new(label)
                                    .monospace()
                                    .color(ui.visuals().error_fg_color),
                            ))
                            .on_hover_ui_at_pointer(label_maker)
                            .on_hover_text_at_pointer({
                                RichText::new(requirements).color(ui.visuals().warn_fg_color)
                            });
                        }
                        None => {
                            ui.monospace(label).on_hover_ui_at_pointer(label_maker);
                        }
                    }

                    let key = State::make_hash(label);

                    let show_password = &mut self.state.show_password;
                    let is_pass = show_password.get(&key).copied().unwrap_or(password);

                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_visible_ui(password, |ui| {
                                let down = ui
                                    .small_button(crate::font_icon::HIDDEN)
                                    .is_pointer_button_down_on();
                                if password {
                                    *show_password.entry(key).or_insert(true) = !down;
                                }
                            });

                            ui.add(TextEdit::singleline(value).password(is_pass));
                        });
                    });

                    ui.end_row()
                }
            });
    }

    fn label_for_name(ui: &mut egui::Ui) {
        ui.label("Your name on Twitch");
        ui.label("This is associated with your OAuth token");
    }

    fn label_for_oauth_token(ui: &mut egui::Ui) {
        ui.label("A OAuth token associated with your name.");
    }

    fn label_for_client_id(ui: &mut egui::Ui) {
        ui.label("Client-Id for using the Twitch Helix API");
    }

    fn label_for_client_secret(ui: &mut egui::Ui) {
        ui.label("Client-Secret associated with the Client-Id");
    }

    fn validate_name(input: &str) -> Validation {
        use Validation::*;
        match () {
            _ if input.is_empty() => CannotBeEmpty,
            _ if input.chars().any(|c| c.is_ascii_whitespace()) => CannotContainSpaces,
            _ => Valid,
        }
    }

    fn validate_oauth_token(input: &str) -> Validation {
        if !input.starts_with("oauth:") {
            return Validation::MustStartWithOAuth;
        }

        match input.len() {
            36 => Validation::Valid,
            n => Validation::MustBeLength(Self::OAUTH_TOKEN_LABEL, 36, n),
        }
    }

    const fn validate_client_id(input: &str) -> Validation {
        match input.len() {
            30 => Validation::Valid,
            n => Validation::MustBeLength(Self::CLIENT_ID_LABEL, 30, n),
        }
    }

    const fn validate_client_secret(input: &str) -> Validation {
        match input.len() {
            30 => Validation::Valid,
            n => Validation::MustBeLength(Self::CLIENT_SECRET_LABEL, 30, n),
        }
    }

    const NAME_LABEL: &'static str = "Name";
    const OAUTH_TOKEN_LABEL: &'static str = "OAuth Token";
    const CLIENT_ID_LABEL: &'static str = "Client-Id";
    const CLIENT_SECRET_LABEL: &'static str = "Client-Secret";

    const LABELS: [(&'static str, fn(&str) -> Validation, fn(&mut egui::Ui)); 4] = [
        (Self::NAME_LABEL, Self::validate_name, Self::label_for_name),
        (
            Self::OAUTH_TOKEN_LABEL,
            Self::validate_oauth_token,
            Self::label_for_oauth_token,
        ),
        (
            Self::CLIENT_ID_LABEL,
            Self::validate_client_id,
            Self::label_for_client_id,
        ),
        (
            Self::CLIENT_SECRET_LABEL,
            Self::validate_client_secret,
            Self::label_for_client_secret,
        ),
    ];
}

#[derive(Copy, Clone)]
enum Validation {
    CannotBeEmpty,
    CannotContainSpaces,
    MustStartWithOAuth,
    MustBeLength(&'static str, usize, usize),
    Valid,
}

impl Validation {
    fn display(&self) -> Option<Cow<'static, str>> {
        let s = match self {
            Self::CannotBeEmpty => "The input cannot be empty".into(),
            Self::CannotContainSpaces => "Spaces are not allowed".into(),
            Self::MustStartWithOAuth => "Token must start with `oauth:`".into(),
            Self::MustBeLength(prefix, length, actual) => {
                format!("{prefix} must be {length} in length. (currently {actual})").into()
            }
            _ => return None,
        };
        Some(s)
    }
}
