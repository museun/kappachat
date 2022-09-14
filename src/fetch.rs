use std::borrow::Cow;

use uuid::Uuid;

use crate::{RequestPaint, TaskQueue};

pub trait FetchUrl {
    fn url(&self) -> Cow<'_, str>;
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TwitchImage {
    id: Uuid,
    url: String,
    kind: TwitchImageKind,
}

impl TwitchImage {
    pub fn emote(id: impl ToString, name: impl ToString, url: impl ToString) -> Self {
        Self {
            id: Uuid::new_v4(),
            url: url.to_string(),
            kind: TwitchImageKind::Emote {
                id: id.to_string(),
                name: name.to_string(),
            },
        }
    }

    pub fn badge(id: impl ToString, set_id: impl ToString, url: impl ToString) -> Self {
        let url = url.to_string();
        let uuid = Self::extract_uuid(&url).expect("uuid in url");
        Self {
            id: uuid,
            url,
            kind: TwitchImageKind::Badge {
                id: id.to_string(),
                set_id: set_id.to_string(),
            },
        }
    }

    fn extract_uuid(input: &str) -> Option<uuid::Uuid> {
        static PATTERN: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
            const PATTERN: &str =
                r#"^.*?(?P<uuid>[A-Fa-f0-9]{8}-(?:[A-Fa-f0-9]{4}-){3}[A-Fa-f0-9]{12}).*?$"#;
            regex::Regex::new(PATTERN).unwrap()
        });

        PATTERN.captures(input)?.name("uuid")?.as_str().parse().ok()
    }

    pub const fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        use TwitchImageKind::*;
        match &self.kind {
            Emote { id: name, .. } | Badge { set_id: name, .. } => name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum TwitchImageKind {
    Emote { id: String, name: String },
    Badge { id: String, set_id: String },
}

impl FetchUrl for TwitchImage {
    fn url(&self) -> Cow<'_, str> {
        (&*self.url).into()
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
    pub fn create(repaint: impl RequestPaint + 'static) -> Self {
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

        fn fetch(
            agent: &ureq::Agent,
            url: &str,
        ) -> anyhow::Result<impl Read + Send + Sync + 'static> {
            Ok(agent.get(url).call()?.into_reader())
        }

        let agent = ureq::agent();
        for item in queue {
            let mut body = match fetch(&agent, &*item.url()) {
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
