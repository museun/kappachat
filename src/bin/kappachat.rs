#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use eframe::{NativeOptions, Storage};

use egui_extras::RetainedImage;
use kappachat::{
    helix, kappas,
    state::{AppState, PersistState},
    EnvConfig, SETTINGS_KEY,
};

const DEFAULT_PIXELS_PER_POINT: f32 = 1.0;
const DEFAULT_IMAGE_SIZE: f32 = 32.0;

fn load_settings(state: &mut PersistState, storage: &dyn Storage) {
    let mut deser = match storage
        .get_string(SETTINGS_KEY)
        .as_deref()
        .map(serde_json::from_str::<PersistState>)
        .transpose()
        .ok()
        .flatten()
    {
        Some(deser) => deser,
        None => return,
    };

    macro_rules! merge {
        ($($binding:tt)*) => {
            $( { state.$binding = deser.$binding } )*
        };
    }

    merge! {
        pixels_per_point
        channels
        key_mapping
        tab_bar_image_size
        tab_bar_position
        show_image_mask
    }

    type Extract = for<'e> fn(&'e mut EnvConfig) -> &'e mut String;

    fn maybe_update<'a: 'e, 'b: 'e, 'e>(
        left: &'a mut EnvConfig,
        right: &'b mut EnvConfig,
        extract: Extract,
    ) {
        let left = extract(left);
        let right = extract(right);
        if left.trim().is_empty() {
            *left = std::mem::take(right)
        }
    }

    for extract in [
        (move |e| &mut e.twitch_oauth_token) as Extract,
        (move |e| &mut e.twitch_name) as Extract,
        (move |e| &mut e.twitch_client_id) as Extract,
        (move |e| &mut e.twitch_client_secret) as Extract,
    ] {
        maybe_update(&mut state.env_config, &mut deser.env_config, extract);
    }
}

fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env", ".secrets.env"]); //".secrets.env"
    let recv = kappachat::init_logger();

    let kappas = kappas::load_kappas();

    let dark_image_mask =
        RetainedImage::from_image_bytes("dark_mask_png", kappachat::DARK_MASK_PNG)
            .expect("load mask");

    let mut state = PersistState {
        env_config: EnvConfig::load_from_env(),
        pixels_per_point: DEFAULT_PIXELS_PER_POINT,
        tab_bar_image_size: DEFAULT_IMAGE_SIZE,
        ..Default::default()
    };

    eframe::run_native(
        kappachat::APP_NAME,
        NativeOptions::default(),
        Box::new(move |cc| {
            if let Some(storage) = cc.storage {
                load_settings(&mut state, storage);
            }

            cc.egui_ctx.set_pixels_per_point(state.pixels_per_point);

            let helix = poll_promise::Promise::spawn_thread("helix_initialization", {
                let config = state.env_config.clone();
                move || {
                    log::trace!("getting helix");
                    let helix = helix::Client::fetch_oauth(
                        &config.twitch_client_id,
                        &config.twitch_client_secret,
                    )
                    .expect("fetch");
                    log::trace!("got helix");
                    helix
                }
            });

            let state = AppState::new(cc.egui_ctx.clone(), kappas, state, helix, dark_image_mask);
            Box::new(kappachat::App::new(cc.egui_ctx.clone(), state, recv))
        }),
    );

    Ok(())
}
