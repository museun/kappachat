use kappachat::twitch;

fn main() -> anyhow::Result<()> {
    let s = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/logs.json"));

    let de = serde_json::de::StreamDeserializer::<'_, _, twitch::Message>::new(
        serde_json::de::StrRead::new(s),
    );

    for item in de.into_iter().flatten() {
        if let Some(msg) = item.as_privmsg() {
            eprintln!(
                "[{target}] {sender}: {data}",
                sender = msg.sender,
                target = msg.target,
                data = msg.data
            );
        }
    }

    // simple_env_load::load_env_from([".dev.env", ".secrets.env"]);

    // let config = EnvConfig::load_from_env()?;

    // let reg = twitch::Registration {
    //     address: "irc.chat.twitch.tv:6667",
    //     nick: &config.twitch_name,
    //     pass: &config.twitch_oauth_token,
    // };
    // let (client, _identity) = twitch::Client::connect(reg)?;

    // let (read_tx, read_rx) = flume::unbounded();
    // let (write_tx, write_rx) = flume::unbounded();

    // struct App {
    //     read_tx: flume::Sender<twitch::Message>,
    //     read_rx: flume::Receiver<twitch::Message>,

    //     write_tx: flume::Sender<String>,
    //     write_rx: flume::Receiver<String>,

    //     twitch: twitch::Twitch,

    //     write: Box<dyn std::io::Write>,
    // }

    // write_tx.send("JOIN #museun,#togglebit\r\n".into()).unwrap();

    // impl eframe::App for App {
    //     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    //         self.twitch.poll(&self.write_rx, &self.read_tx).unwrap();

    //         if let Ok(msg) = self.read_rx.try_recv() {
    //             eprintln!("{}", msg.raw.escape_debug());
    //             writeln!(&mut self.write, "{}", serde_json::to_string(&msg).unwrap()).unwrap();
    //             self.write.flush().unwrap();
    //         }
    //     }
    // }

    // let file = std::fs::File::options()
    //     .append(true)
    //     .create(true)
    //     .write(true)
    //     .open("logs.json")
    //     .unwrap();

    // eframe::run_native(
    //     "gather",
    //     eframe::NativeOptions::default(),
    //     Box::new(|cc| {
    //         let ctx = cc.egui_ctx.clone();
    //         Box::new(App {
    //             read_tx,
    //             read_rx,
    //             write_tx,
    //             write_rx,
    //             twitch: client.spawn_listen(ctx),
    //             write: Box::new(file),
    //         })
    //     }),
    // );

    Ok(())
}
