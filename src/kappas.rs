use egui_extras::RetainedImage;

macro_rules! kappas {
    ($($name:ident => $lit:literal)*) => {
        $( pub const $name: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), $lit)); )*

        pub fn load_kappas() -> Vec<RetainedImage> {
            [$(($name, $lit),)*]
            .into_iter()
            .map(|(data, name)| RetainedImage::from_image_bytes(name, data).unwrap())
            .collect()
        }
    };
}

kappas! {
    KAPPA_PNG          => "/kappas/kappa.png"
    KAPPA_CLAUS_PNG    => "/kappas/kappa_claus.png"
    KAPPA_PRIDE_PNG    => "/kappas/kappa_pride.png"
    KAPPA_ROSS_PNG     => "/kappas/kappa_ross.png"
    KAPPA_WEALTH_PNG   => "/kappas/kappa_wealth.png"
    KAPPA_DARKMODE_PNG => "/kappas/kappa_darkmode.png"
    KAPPA_HD_PNG       => "/kappas/kappa_hd.png"
    KAPPA_KEEPO_PNG    => "/kappas/kappa_keepo.png"
    KAPPA_MINI_PNG     => "/kappas/kappa_mini.png"
}
