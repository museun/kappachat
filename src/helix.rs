use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

pub struct Channel {
    pub id: u64,
    pub login: String,

    pub display_name: String,
    pub description: String,

    pub image_id: Uuid,
    pub profile_image_url: String,
}

#[derive(Debug, Default)]
pub struct Chatters {
    pub count: usize,
    pub chatters: Vec<(Kind, String)>,
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    pub created_at: String,
    pub description: String,
    pub display_name: String,
    pub id: String,
    pub login: String,
    pub profile_image_url: String,
}

pub enum IdOrLogin<'a> {
    Id(&'a str),
    Login(&'a str),
}

#[derive(Clone)]
pub struct Client {
    oauth: Arc<OAuth>,
    agent: ureq::Agent,
}

impl Client {
    pub fn fetch_oauth(client_id: &str, client_secret: &str) -> anyhow::Result<Self> {
        let agent = ureq::agent();
        let oauth = OAuth::create(agent.clone(), client_id, client_secret).map(Arc::new)?;
        Ok(Self { oauth, agent })
    }

    pub fn get_chatters_for(channel: &str) -> anyhow::Result<Chatters> {
        let resp = ureq::get(&format!(
            "https://tmi.twitch.tv/group/user/{channel}/chatters"
        ))
        .call()?
        .into_string()?;

        Self::chatters_json(&resp)
    }

    pub fn get_users<'a>(
        &self,
        logins: impl IntoIterator<Item = IdOrLogin<'a>>,
    ) -> anyhow::Result<Vec<User>> {
        // TODO split on 100 query-pair boundaries
        self.get_response(
            "users",
            logins.into_iter().map(|input| match input {
                IdOrLogin::Id(id) => ("id", id),
                IdOrLogin::Login(login) => ("login", login.strip_prefix('#').unwrap_or(login)),
            }),
        )
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
            .chain(chatters.vips.into_iter().zip(repeat(Vip)))
            .chain(chatters.viewers.into_iter().zip(repeat(Viewer)))
            .map(|(name, kind)| (kind, name))
            .collect();

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
