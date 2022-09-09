use std::borrow::Cow;

#[derive(Debug)]
pub enum Command<'a> {
    Message { raw: &'a str },
    Join { channel: &'a str },
    Part { channel: Option<&'a str> },
    Connect,
    Invalid { raw: &'a str },
}

impl<'a> Command<'a> {
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
        }
    }

    pub fn parse(input: &'a str) -> Self {
        if !input.starts_with('/') {
            return Self::Message { raw: input };
        }

        let mut iter = input[1..].splitn(2, ' ');

        match iter.next().expect("input should be valid here") {
            "join" => {
                if let Some(channel) = iter.next() {
                    Self::Join { channel }
                } else {
                    Self::Invalid { raw: input }
                }
            }

            "part" => Self::Part {
                channel: iter.next(),
            },

            "connect" => Self::Connect,

            _ => Self::Invalid { raw: input },
        }
    }
}
