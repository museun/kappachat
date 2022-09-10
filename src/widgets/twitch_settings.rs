use egui::{text::LayoutJob, Align, Grid, Key, Label, Layout, RichText, TextEdit};

use crate::{ext::JobExt, state::SettingsState, EnvConfig};

pub struct TwitchSettings<'a, 'b> {
    config: &'a mut EnvConfig,
    settings_state: &'b mut SettingsState,
}

impl<'a, 'b> TwitchSettings<'a, 'b> {
    pub fn new(config: &'a mut EnvConfig, settings_state: &'b mut SettingsState) -> Self {
        Self {
            config,
            settings_state,
        }
    }

    fn label_for_name(ui: &mut egui::Ui) {
        ui.label({
            let font_id = crate::get_heading_font_id(ui);
            LayoutJob::default()
                .simple_no_space(
                    "Your name on",
                    font_id.clone(),
                    ui.visuals().strong_text_color(),
                )
                .simple("Twitch", font_id, crate::TWITCH_COLOR)
        });

        ui.label({
            let font_id = crate::get_body_font_id(ui);
            LayoutJob::default()
                .simple_no_space(
                    "This is associated with your",
                    font_id.clone(),
                    ui.visuals().text_color(),
                )
                .simple(
                    "OAuth token",
                    font_id.clone(),
                    ui.visuals().strong_text_color(),
                )
        });
    }

    fn label_for_oauth_token(ui: &mut egui::Ui) {
        ui.label({
            let font_id = crate::get_heading_font_id(ui);
            LayoutJob::default().simple_no_space(
                "A OAuth token associated with your name.",
                font_id.clone(),
                ui.visuals().strong_text_color(),
            )
        });

        ui.label({
            let font_id = crate::get_body_font_id(ui);
            LayoutJob::default()
                .simple_no_space(
                    "This should be in the form of:\n",
                    font_id.clone(),
                    ui.visuals().text_color(),
                )
                .simple_no_space("oauth:", font_id.clone(), ui.visuals().strong_text_color())
                .simple_no_space(
                    "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    font_id.clone(),
                    ui.visuals().weak_text_color(),
                )
        });
    }

    fn label_for_client_id(ui: &mut egui::Ui) {
        ui.label("Client-Id for using the Twitch Helix API");
    }

    fn label_for_client_secret(ui: &mut egui::Ui) {
        ui.label("Client-Secret associated with the Client-Id");
    }

    fn validate_name(input: &str) -> bool {
        !(input.is_empty() || input.chars().any(|c| c.is_ascii_whitespace()))
    }

    fn validate_oauth_token(input: &str) -> bool {
        input.starts_with("oauth:") && input.len() == 36
    }

    const fn validate_client_id(input: &str) -> bool {
        input.len() == 30
    }

    const fn validate_client_secret(input: &str) -> bool {
        Self::validate_client_id(input)
    }

    const LABELS: [(
        &'static str,
        fn(&mut egui::Ui),
        fn(&str) -> bool,
        &'static str,
    ); 4] = [
        (
            "Name", //
            Self::label_for_name as _,
            Self::validate_name,
            "The name cannot contain spaces",
        ),
        (
            "OAuth Token",
            Self::label_for_oauth_token as _,
            Self::validate_oauth_token,
            "The token must start with oauth: and be 36 characters in length",
        ),
        (
            "Client-Id",
            Self::label_for_client_id as _,
            Self::validate_client_id,
            "The client-id must be 30 characters in length",
        ),
        (
            "Client-Secret",
            Self::label_for_client_secret as _,
            Self::validate_client_secret,
            "The client-secret must be 30 characters in length",
        ),
    ];
}

impl<'a, 'b> egui::Widget for TwitchSettings<'a, 'b> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Grid::new("twitch_settings")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for ((label, label_maker, validator, requirements), (value, password)) in
                    Self::LABELS.into_iter().zip([
                        (&mut self.config.twitch_name, false),
                        (&mut self.config.twitch_oauth_token, true),
                        (&mut self.config.twitch_client_id, false),
                        (&mut self.config.twitch_client_secret, true),
                    ])
                {
                    if !validator(&value) {
                        ui.horizontal(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.monospace(crate::font_icon::HELP)
                                    .on_hover_ui_at_pointer(label_maker);

                                ui.add(Label::new(
                                    RichText::new(label)
                                        .monospace()
                                        .color(ui.visuals().error_fg_color),
                                ))
                                .on_hover_text_at_pointer({
                                    let requirements = value
                                        .is_empty()
                                        .then_some(requirements)
                                        .unwrap_or("The input cannot be empty");

                                    RichText::new(requirements).color(ui.visuals().warn_fg_color)
                                });
                            })
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.monospace(crate::font_icon::HELP)
                                    .on_hover_ui_at_pointer(label_maker);

                                ui.monospace(label);
                            });
                        });
                    }

                    TwitchSettingsView {
                        settings_state: self.settings_state,
                        key: SettingsState::make_hash(label),
                        password,
                        view: ui,
                    }
                    .show_entry(value);

                    ui.end_row()
                }
            })
            .response
    }
}

struct TwitchSettingsView<'a, 'u> {
    settings_state: &'a mut SettingsState,
    key: u64,
    password: bool,
    view: &'u mut egui::Ui,
}

impl<'a, 'u> TwitchSettingsView<'a, 'u> {
    fn show_entry(&mut self, val: &'a mut String) {
        let is_pass = self
            .settings_state
            .twitch_visible
            .get(&self.key)
            .copied()
            .unwrap_or(self.password);

        let resp = self.view.add(TextEdit::singleline(val).password(is_pass));

        if self.password {
            let down = self
                .view
                .small_button(crate::font_icon::HIDDEN)
                .is_pointer_button_down_on();

            *self
                .settings_state
                .twitch_visible
                .entry(self.key)
                .or_insert(true) = !down;
        };

        if resp.lost_focus() && self.view.ctx().input().key_pressed(Key::Enter) {}
    }
}
