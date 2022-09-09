use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

#[derive(Debug)]
pub struct KeyMapping {
    map: Vec<(Chord, KeyAction)>,
    reverse: Vec<(KeyAction, Vec<Chord>)>,
}

impl Default for KeyMapping {
    fn default() -> Self {
        use egui::Key::*;
        use KeyAction::*;

        #[rustfmt::skip]
        macro_rules! key {
            (@inner) => { Chord::builder() };
            ($ident:ident) => { key!(@inner).key($ident) };
            (alt $ident:ident) => { key!(@inner).alt().key($ident) };
            (ctrl $ident:ident) => { key!(@inner).ctrl().key($ident) };
        }

        #[rustfmt::skip]
        const DEFAULT: [(Chord, KeyAction); 27] = [
            (key!(F1), ToggleHelp),
            (key!(ctrl L), ToggleLineMode),
            (key!(F2), ToggleTabBar),
            (key!(ctrl T), ToggleTimestamps),
            (key!(ctrl U), ToggleUserList),
            // number row starts at 1
            (key!(ctrl Num0), SwitchTab9), (key!(alt Num0), SwitchTab9),
            (key!(ctrl Num1), SwitchTab0), (key!(alt Num1), SwitchTab0),
            (key!(ctrl Num2), SwitchTab1), (key!(alt Num2), SwitchTab1),
            (key!(ctrl Num3), SwitchTab2), (key!(alt Num3), SwitchTab2),
            (key!(ctrl Num4), SwitchTab3), (key!(alt Num4), SwitchTab3),
            (key!(ctrl Num5), SwitchTab4), (key!(alt Num5), SwitchTab4),
            (key!(ctrl Num6), SwitchTab5), (key!(alt Num6), SwitchTab5),
            (key!(ctrl Num7), SwitchTab6), (key!(alt Num7), SwitchTab6),
            (key!(ctrl Num8), SwitchTab7), (key!(alt Num8), SwitchTab7),
            (key!(ctrl Num9), SwitchTab8), (key!(alt Num9), SwitchTab8),
            (key!(ctrl N), NextTab),
            (key!(ctrl P), PreviousTab),
        ];

        let map = DEFAULT.into_iter().collect::<Vec<_>>();
        let reverse = Self::make_reverse(&map);

        Self { map, reverse }
    }
}

impl KeyMapping {
    fn make_reverse(map: &[(Chord, KeyAction)]) -> Vec<(KeyAction, Vec<Chord>)> {
        map.iter()
            .copied()
            .fold(BTreeMap::<_, Vec<_>>::new(), |mut map, (chord, action)| {
                map.entry(action).or_default().push(chord);
                map
            })
            .into_iter()
            .collect()
    }

    pub fn reverse_mapping(&mut self) -> &mut [(KeyAction, Vec<Chord>)] {
        &mut self.reverse
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &(Chord, KeyAction)> //
           + ExactSizeIterator
           + DoubleEndedIterator
           + '_ {
        self.map.iter()
    }

    pub fn find(&self, key: egui::Key, modifiers: egui::Modifiers) -> Option<KeyAction> {
        let chord = Chord {
            ctrl: modifiers.ctrl,
            alt: modifiers.alt,
            shift: modifiers.shift,
            cmd: modifiers.mac_cmd,
            key,
        };

        self.map
            .iter()
            .find_map(|&(key, action)| (key == chord).then_some(action))
    }
}

impl<'de> ::serde::Deserialize<'de> for KeyMapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = KeyMapping;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "expecting a key mapping")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                use serde::de::Error as _;

                let mut this = Self::Value {
                    map: vec![],
                    reverse: vec![],
                };
                let mut seen = HashSet::<Chord>::new();

                while let Some((val, key)) = map.next_entry_seed(
                    std::marker::PhantomData::<&str>,
                    std::marker::PhantomData::<Vec<&str>>,
                )? {
                    for key in key {
                        let key = key.parse().map_err(A::Error::custom)?;
                        let val: KeyAction = val.parse().map_err(A::Error::custom)?;

                        if !seen.insert(key) {
                            return Err(A::Error::custom(format!(
                                "duplicate entry found: {} -> {}",
                                key.display(),
                                val.display(),
                            )));
                        }

                        this.map.push((key, val));
                    }
                }

                this.reverse = Self::Value::make_reverse(&this.map);
                Ok(this)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl ::serde::Serialize for KeyMapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        use ::serde::ser::SerializeMap as _;

        let mapping =
            self.map
                .iter()
                .fold(BTreeMap::<_, Vec<_>>::new(), |mut map, (chord, action)| {
                    map.entry(action).or_default().push(chord);
                    map
                });

        let mut map = serializer.serialize_map(Some(mapping.len()))?;
        for (k, v) in mapping {
            map.serialize_entry(
                &k.display(),
                &v.iter().map(|c| c.display()).collect::<Vec<_>>(),
            )?;
        }
        map.end()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Chord {
    ctrl: bool,
    alt: bool,
    shift: bool,
    cmd: bool,
    key: egui::Key,
}

impl Chord {
    const fn builder() -> ChordBuilder {
        ChordBuilder {
            ctrl: false,
            alt: false,
            shift: false,
            cmd: false,
        }
    }

    pub fn ctrl(&mut self) -> &mut bool {
        &mut self.ctrl
    }

    pub fn alt(&mut self) -> &mut bool {
        &mut self.alt
    }

    pub fn shift(&mut self) -> &mut bool {
        &mut self.shift
    }

    pub fn cmd(&mut self) -> &mut bool {
        &mut self.cmd
    }

    pub fn display_key(&self) -> &'static str {
        KeyHelper::stringify_key(&self.key)
    }

    pub const fn key(&self) -> egui::Key {
        self.key
    }
}

struct ChordBuilder {
    ctrl: bool,
    alt: bool,
    shift: bool,
    cmd: bool,
}

impl ChordBuilder {
    const fn ctrl(self) -> Self {
        Self { ctrl: true, ..self }
    }

    const fn alt(self) -> Self {
        Self { alt: true, ..self }
    }

    const fn shift(self) -> Self {
        Self {
            shift: true,
            ..self
        }
    }

    const fn cmd(self) -> Self {
        Self { cmd: true, ..self }
    }

    const fn key(self, key: egui::Key) -> Chord {
        Chord {
            ctrl: self.ctrl,
            alt: self.alt,
            shift: self.shift,
            cmd: self.cmd,
            key,
        }
    }
}

impl FromStr for Chord {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (mut ctrl, mut alt, mut shift, mut cmd) = <_>::default();

        macro_rules! check {
            ($key:expr) => {{
                (*match $key {
                    s if s.eq_ignore_ascii_case("cmd") => &mut cmd,
                    s if s.eq_ignore_ascii_case("ctrl") => &mut ctrl,
                    s if s.eq_ignore_ascii_case("shift") => &mut shift,
                    s if s.eq_ignore_ascii_case("alt") => &mut alt,
                    s => return Err(format!("unknown modifier: '{s}'")),
                } = true);
            }};
        }

        let mut input = s;
        while let Some((head, tail)) = input.split_once('+') {
            input = tail.trim();
            check!(head.trim())
        }

        let input = if let Some((mod_, key)) = input.split_once(' ') {
            check!(mod_);
            key.trim()
        } else {
            input.trim()
        };

        KeyHelper::parse_key(input)
            .map(|key| Self {
                ctrl,
                alt,
                shift,
                cmd,
                key,
            })
            .ok_or_else(|| format!("unknown key: {input}"))
    }
}

impl Chord {
    pub fn display(&self) -> String {
        let mut buf = String::new();

        for repr in [
            (self.cmd, "Cmd"),
            (self.ctrl, "Ctrl"),
            (self.alt, "Alt"),
            (self.shift, "Shift"),
        ]
        .into_iter()
        .filter_map(|(c, key)| c.then_some(key))
        {
            if !buf.is_empty() {
                buf.push('+')
            }
            buf.push_str(repr)
        }
        if !buf.is_empty() {
            buf.push('+')
        }

        buf.push_str(KeyHelper::stringify_key(&self.key));
        buf
    }
}

pub struct KeyHelper;
impl KeyHelper {
    pub const fn keys() -> &'static [(&'static str, egui::Key)] {
        use egui::Key::*;
        #[rustfmt::skip]
        const KEYS: &[(&str, egui::Key)] = &[
            ("ArrowDown", ArrowDown), ("ArrowLeft", ArrowLeft), ("ArrowRight", ArrowRight), ("ArrowUp", ArrowUp),
            ("Escape", Escape), ("Tab", Tab), ("Backspace", Backspace), ("Enter", Enter), ("Space", Space),
            ("Insert", Insert), ("Delete", Delete), ("Home", Home), ("End", End), ("PageUp", PageUp), ("PageDown", PageDown),
            ("0", Num0), ("1", Num1), ("2", Num2), ("3", Num3), ("4", Num4), ("5", Num5), ("6", Num6), ("7", Num7), ("8", Num8), ("9", Num9),
            ("A", A), ("B", B), ("C", C), ("D", D), ("E", E), ("F", F), ("G", G), ("H", H), ("I", I), ("J", J), ("K", K), ("L", L), ("M", M),
            ("N", N), ("O", O), ("P", P), ("Q", Q), ("R", R), ("S", S), ("T", T), ("U", U), ("V", V), ("W", W), ("X", X), ("Y", Y), ("Z", Z),
            ("F1", F1), ("F2", F2), ("F3", F3), ("F4", F4), ("F5", F5), ("F6", F6), ("F7", F7), ("F8", F8), ("F9", F9), ("F10", F10), ("F11", F11), ("F12", F12),
            ("F13", F13), ("F14", F14), ("F15", F15), ("F16", F16), ("F17", F17), ("F18", F18), ("F19", F19), ("F20", F20),
        ];
        KEYS
    }

    pub fn parse_key(input: &str) -> Option<egui::Key> {
        Self::keys()
            .iter()
            .find_map(|&(name, key)| (name.eq_ignore_ascii_case(input)).then_some(key))
    }

    pub fn stringify_key(key: &egui::Key) -> &'static str {
        Self::keys()
            .iter()
            .find_map(|(name, k)| (k == key).then_some(*name))
            .unwrap()
    }
}

macro_rules! define_key {
    ($($ident:ident => $help:literal)*) => {
        #[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
        pub enum KeyAction {
            $($ident,)*
        }

        impl FromStr for KeyAction {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let key = match s {
                    $(stringify!($ident) => Self::$ident,)*
                    s => return Err(format!("unknown action: '{s}'")),
                };
                Ok(key)
            }
        }

        impl KeyAction {
            pub const fn display(&self) -> &'static str {
                match self {
                    $(Self::$ident => stringify!($ident),)*
                }
            }

            pub const fn help(&self) -> &'static str {
                match self {
                    $(Self::$ident => $help,)*
                }
            }
        }
    };
}

define_key! {
    ToggleHelp => "Toggles the help window"
    ToggleLineMode => "Cycles through the line modes for the current tab"
    ToggleTabBar => "Toggles the tab bar"
    ToggleTimestamps => "Toggles whether timestamps are visible for the current tab"
    ToggleUserList => "Toggles whether the userlist is visible for the current tab"
    SwitchTab0 => "Switch to Tab #0"
    SwitchTab1 => "Switch to Tab #1"
    SwitchTab2 => "Switch to Tab #2"
    SwitchTab3 => "Switch to Tab #3"
    SwitchTab4 => "Switch to Tab #4"
    SwitchTab5 => "Switch to Tab #5"
    SwitchTab6 => "Switch to Tab #6"
    SwitchTab7 => "Switch to Tab #7"
    SwitchTab8 => "Switch to Tab #8"
    SwitchTab9 => "Switch to Tab #9"
    NextTab => "Switches to the next tab, wrapping around"
    PreviousTab => "Switches to the previous tab, wrapping around"
}
