#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ChatLayout {
    #[default]
    Traditional,
    Modern,
}

impl ChatLayout {
    pub fn cycle(&mut self) {
        *self = match self {
            Self::Traditional => Self::Modern,
            Self::Modern => Self::Traditional,
        };
    }
}
