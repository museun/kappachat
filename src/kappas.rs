use egui_extras::RetainedImage;

pub const KAPPA_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"), //
        "/kappas/kappa.png"
    ));

pub const KAPPA_CLAUS_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/kappas/kappa_claus.png"
    ));

pub const KAPPA_PRIDE_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/kappas/kappa_pride.png"
    ));

pub const KAPPA_ROSS_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/kappas/kappa_ross.png"
    ));

pub const KAPPA_WEALTH_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/kappas/kappa_wealth.png"
    ));

pub fn load_kappas() -> Vec<RetainedImage> {
    [
        (KAPPA_PNG, "kappa.png"),
        (KAPPA_CLAUS_PNG, "kappa_claus.png"),
        (KAPPA_PRIDE_PNG, "kappa_pride.png"),
        (KAPPA_ROSS_PNG, "kappa_ross.png"),
        (KAPPA_WEALTH_PNG, "kappa_wealth.png"),
    ]
    .into_iter()
    .map(|(data, name)| RetainedImage::from_image_bytes(name, data).unwrap())
    .collect()
}
