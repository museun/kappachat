#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use std::collections::HashMap;

use egui::Color32;

pub const TWITCH_COLOR: Color32 = Color32::from_rgb(146, 86, 237);

pub trait RequestPaint: Send + Sync {
    fn request_repaint(&self) {}
}

impl RequestPaint for egui::Context {
    fn request_repaint(&self) {
        Self::request_repaint(self)
    }
}

pub struct NoopRepaint;
impl RequestPaint for NoopRepaint {}

pub mod state;

pub mod widgets;

mod config;
pub use config::EnvConfig;

mod key_mapping;
pub use key_mapping::{Chord, KeyAction, KeyHelper, KeyMapping};

pub mod helix;
pub use helix::CachedImages;

pub mod tabs;
pub use tabs::{Line, Tabs};

mod line;
pub use line::TwitchLine;

mod chat_layout;
pub use chat_layout::ChatLayout;

mod queue;
pub use queue::Queue;

pub mod twitch;

mod ext;
pub use ext::JobExt as _;

mod interaction;
pub use interaction::Interaction;

pub mod kappas;

pub mod font_icon;

mod channel;
pub use channel::Channel;

pub mod app;
pub use app::App;

pub const SETTINGS_KEY: &str = "kappa_chat_settings";

// let width = ui.fonts().glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');
// ui.spacing_mut().item_spacing.x = width;

pub trait FetchUrl {
    fn url(&self) -> std::borrow::Cow<'_, str>;
}

struct TaskQueue<I> {
    queue: flume::Sender<I>,
    ready: flume::Receiver<(I, Vec<u8>)>,
    handle: std::thread::JoinHandle<()>,
}

impl<I> TaskQueue<I>
where
    I: Send + Sync + 'static,
{
    pub fn new<
        R: RequestPaint + 'static,
        F: FnOnce(R, flume::Receiver<I>, flume::Sender<(I, Vec<u8>)>) + Send + Sync + 'static,
    >(
        repaint: R,
        spawn: F,
    ) -> Self {
        let (queue_tx, queue_rx) = flume::unbounded();
        let (ready_tx, ready_rx) = flume::unbounded();

        let handle = std::thread::spawn(move || spawn(repaint, queue_rx, ready_tx));

        Self {
            queue: queue_tx,
            ready: ready_rx,
            handle,
        }
    }

    pub fn join(self) -> Vec<(I, Vec<u8>)> {
        drop(self.queue);
        let _ = self.handle.join();
        self.ready.into_iter().collect()
    }

    pub fn enqueue(&self, item: I) {
        let _ = self.queue.send(item);
    }

    pub fn try_next(&self) -> Option<(I, Vec<u8>)> {
        self.ready.try_recv().ok()
    }
}

#[derive(Debug)]
pub enum TwitchImage {
    Emote {
        id: String,
        name: String,
        url: String,
    },
}

impl TwitchImage {
    pub fn id(&self) -> &str {
        match self {
            Self::Emote { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Emote { name, .. } => name,
        }
    }
}

impl FetchUrl for TwitchImage {
    fn url(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::Emote { url, .. } => url.into(),
        }
    }
}

pub struct FetchQueue<I> {
    queue: TaskQueue<I>,
}

impl<I> FetchQueue<I>
where
    I: Send + Sync + 'static,
    I: FetchUrl,
    I: std::fmt::Debug,
{
    pub fn new(repaint: impl RequestPaint + 'static) -> Self {
        Self {
            queue: TaskQueue::new(repaint, Self::spawn),
        }
    }

    pub fn fetch(&self, item: I) {
        self.queue.enqueue(item)
    }

    pub fn try_next(&self) -> Option<(I, Vec<u8>)> {
        self.queue.try_next()
    }

    pub fn join(self) -> Vec<(I, Vec<u8>)> {
        self.queue.join()
    }

    fn spawn(
        repaint: impl RequestPaint + 'static,
        queue: flume::Receiver<I>,
        ready: flume::Sender<(I, Vec<u8>)>,
    ) where
        I: std::fmt::Debug,
    {
        use std::io::Read;

        // TODO pre-allocate the vec
        fn fetch(
            agent: &ureq::Agent,
            url: &str,
        ) -> anyhow::Result<impl std::io::Read + Send + Sync + 'static> {
            Ok(agent.get(url).call()?.into_reader())
        }

        // fn make_dir(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        //     let path = path.as_ref();
        //     let path = if path.is_file() {
        //         path.ancestors().nth(1).unwrap_or(path)
        //     } else {
        //         path
        //     };
        //     std::fs::create_dir_all(path)
        // }

        // let root = PathBuf::from("./data");
        // let _ = std::fs::create_dir_all(&root);

        let agent = ureq::agent();
        for item in queue {
            // let dir = item.path(&root);
            // let file = dir.join(item.name());

            // if let Ok(md) = std::fs::metadata(&file) {
            //     if md.is_file() {
            //         continue;
            //     }
            // }

            // if make_dir(&dir).is_err() {
            //     continue;
            // }

            eprintln!("fetching: {item:?}");

            let mut body = match fetch(&agent, &*item.url()) {
                Ok(body) => body,
                _ => {
                    eprintln!("cannot fetch: {item:?}");
                    continue;
                }
            };

            let mut data = vec![];
            let _ = body.read_to_end(&mut data);
            // let _ = std::fs::write(&file, &mut data);
            let _ = ready.send((item, data));

            eprintln!("repaiting");
            repaint.request_repaint();
        }

        eprintln!("end of fetch loop")
    }
}

pub fn emote_map() -> HashMap<String, String> {
    serde_json::de::StreamDeserializer::<_, [String; 2]>::new(serde_json::de::StrRead::new(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/emotes.json")),
    ))
    .into_iter()
    .flatten()
    .map(|[a, b]| (a, b))
    .collect()
}
