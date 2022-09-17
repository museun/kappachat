use std::{collections::HashSet, hash::Hash};

use uuid::Uuid;

use crate::{
    store::{Image, ImageStore},
    RequestPaint, TaskQueue,
};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ImageKind {
    Emote = 0,
    Badge = 1,
    Display = 2,
}

impl ImageKind {
    pub const fn to_tag(self) -> u8 {
        self as _
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Emote => "Emote",
            Self::Badge => "Badge",
            Self::Display => "Display",
        }
    }

    pub const fn from_tag(id: u8) -> Option<Self> {
        Some(match id {
            0 => Self::Emote,
            1 => Self::Badge,
            2 => Self::Display,
            _ => return None,
        })
    }
}

pub trait FetchImage {
    fn id(&self) -> Uuid;
    fn url(&self) -> &str;
    fn kind(&self) -> ImageKind;
}

fn extract_uuid(input: &str) -> Option<uuid::Uuid> {
    static PATTERN: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
        const PATTERN: &str =
            r#"^.*?(?P<uuid>[A-Fa-f0-9]{8}-(?:[A-Fa-f0-9]{4}-){3}[A-Fa-f0-9]{12}).*?$"#;
        regex::Regex::new(PATTERN).unwrap()
    });

    PATTERN.captures(input)?.name("uuid")?.as_str().parse().ok()
}

pub struct FetchQueue<I>
where
    I: FetchImage,
{
    queue: TaskQueue<I>,
    seen: HashSet<Uuid>,
}

impl<I> FetchQueue<I>
where
    I: Send + Sync + 'static,
    I: FetchImage + std::fmt::Debug,
{
    pub fn create(repaint: impl RequestPaint + 'static) -> Self {
        Self {
            queue: TaskQueue::new(repaint, Self::spawn),
            seen: HashSet::new(),
        }
    }

    pub fn fetch(&mut self, item: I) -> bool {
        if !self.seen.insert(item.id()) {
            return false;
        }

        self.queue.enqueue(item);
        true
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

        fn fetch(
            agent: &ureq::Agent,
            url: &str,
        ) -> anyhow::Result<impl Read + Send + Sync + 'static> {
            Ok(agent.get(url).call()?.into_reader())
        }

        let agent = ureq::agent();
        for item in queue {
            if let Some(img) = ImageStore::<Image>::get::<()>(item.id()) {
                let _ = ready.send((item, img.data.to_vec()));
                continue;
            }

            eprintln!("sending req for {}", item.id());

            let mut body = match fetch(&agent, item.url()) {
                Ok(body) => body,
                _ => {
                    eprintln!("cannot fetch: {item:?}");
                    continue;
                }
            };

            // TODO pre-allocate the vec
            let mut data = vec![];
            let _ = body.read_to_end(&mut data);
            let _ = ready.send((item, data));
            repaint.request_repaint();
        }

        eprintln!("end of fetch loop")
    }
}
