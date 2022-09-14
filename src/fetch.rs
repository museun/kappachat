use crate::{RequestPaint, TaskQueue};

pub trait FetchUrl {
    fn url(&self) -> std::borrow::Cow<'_, str>;
}

#[derive(Debug)]
pub enum TwitchImage {
    Emote {
        id: String,
        name: String,
        url: String,
    },
    Badge {
        id: String, // set?
        name: String,
        url: String,
    },
}

impl TwitchImage {
    pub fn id(&self) -> &str {
        match self {
            Self::Emote { id, .. } | Self::Badge { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Emote { name, .. } | Self::Badge { name, .. } => name,
        }
    }
}

impl FetchUrl for TwitchImage {
    fn url(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::Emote { url, .. } | Self::Badge { url, .. } => url.into(),
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
