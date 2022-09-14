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

pub const KAPPA_DARKMODE_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/kappas/kappa_darkmode.png"
    ));

pub const KAPPA_HD_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"), //
        "/kappas/kappa_hd.png"
    ));

pub const KAPPA_KEEPO_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/kappas/kappa_keepo.png"
    ));

pub const KAPPA_MINI_PNG: &[u8] = //
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/kappas/kappa_mini.png"
    ));

pub fn load_kappas() -> Vec<RetainedImage> {
    [
        (KAPPA_PNG, "kappa.png"),
        (KAPPA_CLAUS_PNG, "kappa_claus.png"),
        (KAPPA_PRIDE_PNG, "kappa_pride.png"),
        (KAPPA_ROSS_PNG, "kappa_ross.png"),
        (KAPPA_WEALTH_PNG, "kappa_wealth.png"),
        (KAPPA_DARKMODE_PNG, "kappa_darkmode.png"),
        (KAPPA_HD_PNG, "kappa_hd.png"),
        (KAPPA_KEEPO_PNG, "kappa_keepo.png"),
        (KAPPA_MINI_PNG, "kappa_mini.png"),
    ]
    .into_iter()
    .map(|(data, name)| RetainedImage::from_image_bytes(name, data).unwrap())
    .collect()
}
