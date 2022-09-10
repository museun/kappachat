use std::{
    borrow::Cow,
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{Shutdown, TcpStream},
    str::FromStr,
};

use anyhow::Context;
use flume::Receiver;

pub const TWITCH_COLORS: [Color; 15] = [
    Color(0x00, 0x00, 0xFF), //
    Color(0x8A, 0x2B, 0xE2), //
    Color(0x5F, 0x9E, 0xA0), //
    Color(0xD2, 0x69, 0x1E), //
    Color(0xFF, 0x7F, 0x50), //
    Color(0x1E, 0x90, 0xFF), //
    Color(0xB2, 0x22, 0x22), //
    Color(0xDA, 0xA5, 0x20), //
    Color(0x00, 0x80, 0x00), //
    Color(0xFF, 0x69, 0xB4), //
    Color(0xFF, 0x45, 0x00), //
    Color(0xFF, 0x00, 0x00), //
    Color(0x2E, 0x8B, 0x57), //
    Color(0x00, 0xFF, 0x7F), //
    Color(0xAD, 0xFF, 0x2F),
];

pub struct Registration<'a> {
    pub address: &'a str,
    pub nick: &'a str,
    pub pass: &'a str,
}

pub enum Status {
    Connecting,
    Connected,
    Registering,
    WaitingForReady,
    Ready,
}

pub struct Client {
    buf: String,
    read: BufReader<TcpStream>,
    write: TcpStream,
}

impl Client {
    pub fn connect(reg: Registration<'_>) -> anyhow::Result<(Self, Identity)> {
        let mut write = TcpStream::connect(reg.address)?;

        for registration in [
            "CAP REQ :twitch.tv/membership\r\n",
            "CAP REQ :twitch.tv/tags\r\n",
            "CAP REQ :twitch.tv/commands\r\n",
            &format!("PASS {}\r\n", reg.pass),
            &format!("NICK {}\r\n", reg.nick),
        ] {
            write.write_all(registration.as_bytes())?
        }
        write.flush()?;

        let buf = String::with_capacity(1024);
        let read = write.try_clone().map(BufReader::new)?;

        let mut this = Self { buf, read, write };

        let identity = this.wait_for_ready()?;

        Ok((this, identity))
    }

    fn wait_for_ready(&mut self) -> anyhow::Result<Identity> {
        loop {
            let msg = self.read_line()?;
            if let Command::Ready = msg.command {
                let user_name = msg
                    .tags
                    .get("display-name")
                    .map(ToString::to_string)
                    .with_context(|| "missing display-name")?;

                let user_id = msg
                    .tags
                    .get_parsed("user-id")
                    .with_context(|| "missing user-id")??;

                let color = msg
                    .tags
                    .get_parsed("color")
                    .with_context(|| "missing color")??;

                let identity = Identity {
                    user_name,
                    user_id,
                    color,
                };

                return Ok(identity);
            }
        }
    }

    fn read_line(&mut self) -> anyhow::Result<Message> {
        self.buf.clear();

        // this blocks
        let n = self.read.read_line(&mut self.buf)?;
        anyhow::ensure!(n > 0, "unexpected eof");

        let msg = Message::parse(&self.buf[..n])?;
        match &msg.command {
            Command::Ping => {
                let token = msg.data.as_ref().with_context(|| "missing token")?;
                self.write
                    .write_all(format!("PONG {token}\r\n").as_bytes())?;
            }
            Command::Error => {
                let message = msg.data.as_ref().with_context(|| "missing message")?;
                anyhow::bail!("twitch error: {message}")
            }
            _ => {}
        };

        Ok(msg)
    }

    pub fn spawn_listen(self, repaint: impl crate::RequestPaint + 'static) -> Twitch {
        let (tx, rx) = flume::unbounded();
        let mut this = self;
        let write = this.write.try_clone().expect("clone");

        let handle = std::thread::spawn(move || {
            loop {
                let msg = this.read_line()?;
                repaint.request_repaint();

                if let Err(..) = tx.send(msg) {
                    break;
                }
            }
            anyhow::Result::<_, anyhow::Error>::Ok(())
        });

        Twitch {
            recv: rx,
            stream: write,
            handle,
        }
    }
}

pub struct Twitch {
    recv: Receiver<Message>,
    stream: TcpStream,
    handle: std::thread::JoinHandle<anyhow::Result<()>>,
}

impl Twitch {
    pub fn poll(
        &self,
        writer: &flume::Receiver<String>,
        reader: &flume::Sender<Message>,
    ) -> anyhow::Result<()> {
        if let Ok(msg) = self.recv.try_recv() {
            reader.send(msg)?;
        }

        if let Ok(data) = writer.try_recv() {
            { &self.stream }.write_all(data.as_bytes())?;
            if !data.ends_with("\r\n") {
                { &self.stream }.write_all(b"\r\n")?;
            }
            eprintln!("-> {}", data.escape_debug());
            { &self.stream }.flush()?;
        }

        Ok(())
    }

    pub fn quit(self) {
        let _ = { &self.stream }.write(b"QUIT\r\n");
        let _ = { &self.stream }.flush();
        let _ = { &self.stream }.shutdown(Shutdown::Both);
        let _ = self.handle.join();
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Color(pub u8, pub u8, pub u8);

impl From<Color> for egui::Color32 {
    fn from(Color(r, g, b): Color) -> Self {
        Self::from_rgb(r, g, b)
    }
}

impl From<&Color> for egui::Color32 {
    fn from(&Color(r, g, b): &Color) -> Self {
        Self::from_rgb(r, g, b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(0xFF, 0xFF, 0xFF)
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let s = match s.len() {
            7 => &s[1..],
            6 => s,
            _ => anyhow::bail!("not a color triplet"),
        };

        let color = u32::from_str_radix(s, 16)?;
        Ok(Self(
            ((color >> 16) & 0xFF) as _,
            ((color >> 8) & 0xFF) as _,
            (color & 0xFF) as _,
        ))
    }
}

#[derive(Debug)]
pub struct Identity {
    pub user_name: String,
    pub user_id: i64,
    pub color: Color,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Tags {
    inner: HashMap<String, String>,
}

impl Tags {
    fn parse(input: &mut &str) -> Option<Self> {
        if !input.starts_with('@') {
            return None;
        }

        let (head, tail) = input.split_once(' ')?;
        *input = tail;

        let inner = head[1..]
            .split_terminator(';')
            .flat_map(|tag| tag.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Some(Self { inner })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|s| &**s)
    }

    pub fn get_parsed<T>(&self, key: &str) -> Option<anyhow::Result<T>>
    where
        T: FromStr,
        T::Err: Into<anyhow::Error>,
    {
        self.get(key)
            .map(<str>::parse)
            .map(|r| r.map_err(Into::into))
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Ready,
    Ping,
    Join,
    Part,
    Privmsg,
    Error,
    Other,
}

impl Command {
    fn parse(input: &mut &str) -> Self {
        let (head, tail) = input.split_at(input.find(' ').unwrap_or(input.len()));
        *input = tail;
        match head {
            "GLOBALUSERSTATE" => Self::Ready,
            "PING" => Self::Ping,
            "JOIN" => Self::Join,
            "PART" => Self::Part,
            "PRIVMSG" => Self::Privmsg,
            "ERROR" => Self::Error,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Prefix {
    User { name: String },
    Server { host: String },
    Empty,
}

impl Prefix {
    pub fn as_user(&self) -> Option<&str> {
        match self {
            Self::User { name } => Some(name),
            _ => None,
        }
    }

    fn parse(input: &mut &str) -> anyhow::Result<Self> {
        if !input.starts_with(':') {
            return Ok(Self::Empty);
        }

        let (head, tail) = input[1..]
            .split_once(' ')
            .with_context(|| "incomplete message")?;

        *input = tail;

        Ok(head
            .split_once('!')
            .map(|(head, ..)| Self::User {
                name: head.to_string(),
            })
            .unwrap_or(Self::Server {
                host: head.to_string(),
            }))
    }
}

#[derive(Debug)]
pub struct Join<'a> {
    pub channel: &'a str,
    pub user: &'a str,
}

#[derive(Debug)]
pub struct Part<'a> {
    pub channel: &'a str,
    pub user: &'a str,
}

#[derive(Debug)]
pub struct Privmsg<'a> {
    pub target: &'a str,
    pub sender: &'a str,
    pub data: &'a str,
    pub tags: &'a Tags,
}

impl<'a> Privmsg<'a> {
    pub fn color(&self) -> Color {
        self.tags
            .get_parsed("color")
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    pub fn emote_span(&self) -> Vec<TextKind<'_>> {
        let (mut list, cursor) = self
            .tags
            .get("emotes")
            .into_iter()
            .flat_map(|s| s.split('/'))
            .flat_map(|c| c.split_once(':'))
            .flat_map(|(emote, range)| {
                emote
                    .parse()
                    .ok()
                    .map(TextKind::Emote)
                    .map(|emote| (emote, range))
            })
            .flat_map(|(emote, range)| {
                range
                    .split(',')
                    .flat_map(|c| c.split_once('-'))
                    .flat_map(|(start, end)| Some((start.parse().ok()?, end.parse().ok()?)))
                    .zip(std::iter::repeat(emote))
                    .map(|(r, e)| (e, r))
            })
            .fold(
                (Vec::new(), 0),
                |(mut v, cursor), (emote, (start, end)): (_, (_, usize))| {
                    if start != cursor {
                        v.push(TextKind::Text(self.data[cursor..start].into()));
                    }
                    v.push(emote);
                    (v, end + 1)
                },
            );

        if cursor != self.data.len() {
            list.push(TextKind::Text(self.data[cursor..].into()))
        }

        list
    }
}

#[derive(Clone, Debug)]
pub enum TextKind<'a> {
    Emote(usize),
    Text(Cow<'a, str>),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub tags: Tags,
    pub prefix: Prefix,
    pub command: Command,
    pub args: Vec<String>,
    pub data: Option<String>,
    pub raw: String,
}

impl Message {
    pub fn as_join(&self) -> Option<Join<'_>> {
        if !matches!(self.command, Command::Join) {
            return None;
        }

        Some(Join {
            channel: self.args.get(0)?,
            user: self.prefix.as_user()?,
        })
    }

    pub fn as_part(&self) -> Option<Part<'_>> {
        if !matches!(self.command, Command::Part) {
            return None;
        }

        Some(Part {
            channel: self.args.get(0)?,
            user: self.prefix.as_user()?,
        })
    }

    pub fn as_privmsg(&self) -> Option<Privmsg<'_>> {
        if !matches!(self.command, Command::Privmsg) {
            return None;
        }

        Some(Privmsg {
            target: self.args.get(0)?,
            sender: self.prefix.as_user()?,
            data: self.data.as_deref()?,
            tags: &self.tags,
        })
    }

    pub(crate) fn parse(input: &str) -> anyhow::Result<Self> {
        let raw = input;
        eprintln!("<- {}", raw.escape_debug());

        let input = &mut input.trim();
        let tags = input.starts_with('@').then(|| Tags::parse(input)).flatten();

        let prefix = Prefix::parse(input)?;
        let command = Command::parse(input);

        let (head, tail) = input.split_at(input.find(':').unwrap_or(input.len()));
        *input = tail;

        let args: Vec<_> = head
            .split_ascii_whitespace()
            .map(ToString::to_string)
            .collect();

        let data = input
            .trim_end()
            .strip_prefix(':')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(Self {
            tags: tags.unwrap_or_default(),
            prefix,
            command,
            args,
            data,
            raw: raw.to_string(),
        })
    }
}
