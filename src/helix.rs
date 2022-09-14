use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use egui_extras::RetainedImage;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Chatters {
    pub count: usize,
    pub chatters: BTreeMap<Kind, Vec<String>>,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Kind {
    Broadcaster,
    Vip,
    Moderator,
    Staff,
    Admin,
    GlobalMod,
    Viewer,
}

impl Kind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Broadcaster => "broadcasters",
            Self::Vip => "vips",
            Self::Moderator => "moderators",
            Self::Staff => "staffs",
            Self::Admin => "admins",
            Self::GlobalMod => "global mods",
            Self::Viewer => "viewers",
        }
    }

    pub fn parse(str: &str) -> Option<Self> {
        Some(match str {
            "broadcaster" => Self::Broadcaster,
            "vip" => Self::Vip,
            "moderator" => Self::Moderator,
            "staff" => Self::Staff,
            "admin" => Self::Admin,
            "global_mod" => Self::GlobalMod,
            "viewer" => Self::Viewer,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default, ::serde::Deserialize)]
struct OAuth {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,

    #[serde(default)]
    client_id: String,

    #[serde(skip)]
    bearer_token: String,
}

impl OAuth {
    pub fn create(
        agent: ureq::Agent,
        client_id: &str,
        client_secret: &str,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(!client_id.is_empty(), "twitch client id was empty");
        anyhow::ensure!(!client_secret.is_empty(), "twitch client secret was empty");

        let req = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials"),
        ]
        .into_iter()
        .fold(
            agent.post("https://id.twitch.tv/oauth2/token"),
            |req, (k, v)| req.query(k, v),
        );

        let resp = req.call()?.into_json();
        Ok(resp.map(|this: Self| Self {
            client_id: client_id.to_string(),
            bearer_token: format!("Bearer {}", this.access_token),
            ..this
        })?)
    }
}

pub struct Client {
    oauth: OAuth,
    agent: ureq::Agent,
}

impl Client {
    pub fn fetch_oauth(client_id: &str, client_secret: &str) -> anyhow::Result<Self> {
        let agent = ureq::agent();
        let oauth = OAuth::create(agent.clone(), client_id, client_secret)?;
        Ok(Self { oauth, agent })
    }

    pub fn get_chatters_for(&self, channel: &str) -> anyhow::Result<Chatters> {
        let resp = self
            .agent
            .get(&format!(
                "https://tmi.twitch.tv/group/user/{channel}/chatters"
            ))
            .call()?
            .into_string()?;

        Self::chatters_json(&resp)
    }

    pub fn get_emotes(&self) -> anyhow::Result<Vec<Emotes>> {
        self.get_response("chat/emotes/global", [])
    }

    pub fn get_badges(&self) -> anyhow::Result<Vec<Badges>> {
        self.get_response("chat/badges/global", [])
    }

    pub fn get_stream_for(&self, channel: &str) -> anyhow::Result<Stream> {
        let mut streams =
            self.get_response("streams", [("user_login", channel), ("first", "1")])?;
        Ok(streams.remove(0))
    }

    fn get_response<'k, 'v, T>(
        &self,
        ep: &str,
        query: impl IntoIterator<Item = (&'k str, &'v str)>,
    ) -> anyhow::Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Resp<T> {
            data: Vec<T>,
        }

        let req = self.agent.get(&format!("https://api.twitch.tv/helix/{ep}"));
        let req = query.into_iter().fold(req, |req, (k, v)| req.query(k, v));

        let req = [
            ("client-id", &self.oauth.client_id), //
            ("authorization", &self.oauth.bearer_token),
        ]
        .into_iter()
        .fold(req, |req, (k, v)| req.set(k, v));

        #[cfg(feature = "save_http_json")]
        let resp: Resp<T> = {
            let s = req.call()?.into_string()?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let ep = ep.replace('/', "_");
            let path = format!("{ep}_{now}.json");
            eprintln!("saving json to: {path}");
            std::fs::write(path, &s).unwrap();
            serde_json::from_str(&s)?
        };

        #[cfg(not(feature = "save_http_json"))]
        let resp: Resp<T> = req.call()?.into_json()?;

        Ok(resp.data)
    }

    pub fn chatters_json(json: &str) -> anyhow::Result<Chatters> {
        #[derive(serde::Deserialize)]
        struct Chatters {
            admins: Vec<String>,
            broadcaster: Vec<String>,
            global_mods: Vec<String>,
            moderators: Vec<String>,
            staff: Vec<String>,
            viewers: Vec<String>,
            vips: Vec<String>,
        }

        #[derive(serde::Deserialize)]
        struct Resp {
            #[serde(rename = "chatter_count")]
            count: usize,
            chatters: Chatters,
        }

        let resp: Resp = serde_json::from_str(json)?;

        use self::Kind::*;
        use std::iter::repeat;

        let (count, chatters) = (resp.count, resp.chatters);
        let head = chatters.admins.into_iter().zip(repeat(Admin));
        let chatters = head
            .chain(chatters.broadcaster.into_iter().zip(repeat(Broadcaster)))
            .chain(chatters.global_mods.into_iter().zip(repeat(GlobalMod)))
            .chain(chatters.moderators.into_iter().zip(repeat(Moderator)))
            .chain(chatters.staff.into_iter().zip(repeat(Staff)))
            .chain(chatters.viewers.into_iter().zip(repeat(Viewer)))
            .chain(chatters.vips.into_iter().zip(repeat(Vip)))
            .fold(
                <BTreeMap<Kind, Vec<String>>>::new(),
                |mut map, (name, kind)| {
                    map.entry(kind).or_default().push(name);
                    map
                },
            );

        Ok(self::Chatters { count, chatters })
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Stream {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub title: String,
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Emotes {
    pub format: Vec<String>,
    pub id: String,
    pub images: HashMap<String, String>,
    pub name: String,
    pub scale: Vec<String>,
    pub theme_mode: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Badges {
    pub set_id: String,
    pub versions: Vec<Versions>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Versions {
    pub id: String,
    pub image_url_1x: String,
    pub image_url_2x: String,
    pub image_url_4x: String,
}

pub struct CachedImages {
    agent: ureq::Agent,
    pub map: HashMap<Uuid, Arc<RetainedImage>>,
    pub id_map: HashMap<Kind, Uuid>,
    pub emote_map: HashMap<usize, Uuid>,
}

impl Default for CachedImages {
    fn default() -> Self {
        Self {
            agent: ureq::agent(),
            map: Default::default(),
            id_map: Default::default(),
            emote_map: Default::default(),
        }
    }
}

impl CachedImages {
    pub fn load_from(path: impl AsRef<Path>) -> Self {
        let map = std::fs::read_dir(path)
            .ok()
            .into_iter()
            .flat_map(|dir| {
                dir.into_iter()
                    .flatten()
                    .filter_map(|c| c.file_type().ok().filter(|c| c.is_file()).map(|_| c.path()))
                    .map(|path| {
                        let id = path.file_stem().unwrap().to_string_lossy();
                        let id = uuid::Uuid::parse_str(&id).unwrap();

                        let data = RetainedImage::from_image_bytes(
                            id.to_string(),
                            &*std::fs::read(&path).unwrap(),
                        )
                        .map(Arc::new)
                        .unwrap();

                        (id, data)
                    })
            })
            .collect();

        Self {
            map,
            agent: ureq::agent(),
            id_map: HashMap::new(),
            emote_map: HashMap::new(),
        }
    }

    pub fn merge_emotes(&mut self, iter: impl IntoIterator<Item = (usize, Uuid)>) {
        self.emote_map.extend(iter)
    }

    pub fn merge_badges(&mut self, badges: &[Badges]) {
        let (tx, rx) = flume::unbounded();
        let (kind_tx, kind_rx) = flume::unbounded();

        for badges in badges.chunks(6) {
            let tx = tx.clone();
            std::thread::scope(|s| {
                for (url, kind) in badges
                    .iter()
                    .flat_map(|badge| {
                        badge
                            .versions
                            .iter()
                            .zip(std::iter::repeat(Kind::parse(&badge.set_id)))
                    })
                    .map(|(v, kind)| (&v.image_url_4x, kind))
                {
                    if let Some(uuid) = Self::extract_uuid(url) {
                        if let Some(kind) = kind {
                            let _ = kind_tx.send((kind, uuid));
                        }
                        if self.map.contains_key(&uuid) {
                            continue;
                        }
                    }

                    s.spawn(|| {
                        if let Ok(res) = self.fetch(url) {
                            let _ = tx.send(res);
                        }
                    });
                }
            });
        }

        drop(tx);
        drop(kind_tx);
        for (uuid, img) in rx {
            let _ = self.map.insert(uuid, img);
        }
        for (kind, uuid) in kind_rx {
            let _ = self.id_map.insert(kind, uuid);
        }
    }

    fn fetch(&self, url: &str) -> anyhow::Result<(Uuid, Arc<RetainedImage>)> {
        let uuid = Self::extract_uuid(url).with_context(|| "invalid url")?;

        eprintln!("fetching: {url}");

        let mut buf = Vec::with_capacity(32 * 3);
        let n = self
            .agent
            .get(url)
            .call()?
            .into_reader()
            .read_to_end(&mut buf)?;

        let data = &buf[..n];
        let img = RetainedImage::from_image_bytes(uuid.to_string(), data)
            .map_err(|err| anyhow::anyhow!("load image: {err}"))?;

        std::fs::write(PathBuf::from("./data").join(uuid.to_string()), data)?;

        Ok((uuid, Arc::new(img)))
    }

    fn extract_uuid(input: &str) -> Option<uuid::Uuid> {
        static PATTERN: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
            const PATTERN: &str =
                r#"^.*?(?P<uuid>[A-Fa-f0-9]{8}-(?:[A-Fa-f0-9]{4}-){3}[A-Fa-f0-9]{12}).*?$"#;
            regex::Regex::new(PATTERN).unwrap()
        });

        PATTERN.captures(input)?.name("uuid")?.as_str().parse().ok()
    }

    pub fn get(&mut self, id: uuid::Uuid) -> Option<Arc<RetainedImage>> {
        self.map.get(&id).map(Arc::clone)
    }
}
