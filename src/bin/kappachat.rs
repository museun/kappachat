#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use eframe::{NativeOptions, Storage};

use kappachat::{
    kappas,
    state::{AppState, PersistState},
    EnvConfig, SETTINGS_KEY,
};

const DEFAULT_PIXELS_PER_POINT: f32 = 1.0;

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

    state.pixels_per_point = deser.pixels_per_point;
    state.channels = deser.channels;
    state.key_mapping = deser.key_mapping;

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

    // let mut cached = CachedImages::load_from("./data");

    // let json = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "badges.json"));
    // #[derive(serde::Deserialize)]
    // struct Resp<T> {
    //     data: Vec<T>,
    // }
    // let badges = serde_json::from_str::<Resp<helix::Badges>>(json)
    //     .unwrap()
    //     .data;

    // cached.merge_badges(&badges);

    let kappas = kappas::load_kappas();

    let mut state = PersistState {
        env_config: EnvConfig::load_from_env(),
        pixels_per_point: DEFAULT_PIXELS_PER_POINT,
        ..Default::default()
    };

    eframe::run_native(
        "KappaChat",
        NativeOptions::default(),
        Box::new(move |cc| {
            if let Some(storage) = cc.storage {
                load_settings(&mut state, storage);
            }

            cc.egui_ctx.set_pixels_per_point(state.pixels_per_point);

            let state = AppState::new(kappas, state);
            Box::new(kappachat::App::new(cc.egui_ctx.clone(), state))
        }),
    );

    Ok(())
}
