use std::borrow::Cow;

#[derive(Debug)]
pub enum Command<'a> {
    Message { raw: &'a str },
    Join { channel: &'a str },
    Part { channel: Option<&'a str> },
    Connect,
    Nothing,
    Invalid { raw: &'a str },
}

impl<'a> Command<'a> {
    // TODO redo this
    pub fn report(&self) -> Cow<'a, str> {
        match self {
            Self::Message { raw } => (*raw).into(),

            Self::Join { channel } => format!("/join {channel}").into(),

            Self::Part {
                channel: Some(channel),
            } => format!("/part {channel}").into(),

            Self::Part { .. } => "/part".into(),

            Self::Connect => "/connect".into(),

            Self::Invalid { raw } => (*raw).into(),

            Self::Nothing => Cow::Borrowed("nothing"),
        }
    }

    pub fn parse(input: &'a str) -> Self {
        if !input.starts_with('/') {
            return Self::Message { raw: input };
        }

        let mut iter = input[1..].splitn(2, ' ');

        match iter.next().expect("input should be valid here") {
            "join" => iter
                .next()
                .map(|channel| Self::Join { channel })
                .unwrap_or(Self::Invalid { raw: input }),

            "part" => Self::Part {
                channel: iter.next(),
            },

            // TODO get rid of this (we're using a GUI for it)
            "connect" => Self::Connect,

            _ => Self::Invalid { raw: input },
        }
    }
}
