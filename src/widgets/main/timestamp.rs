use time::OffsetDateTime;

#[derive(Clone)]
pub struct Timestamp {
    pub(crate) date_time: OffsetDateTime, // why?
    pub(crate) repr: String,
}

impl Timestamp {
    pub fn now_local() -> Self {
        let date_time = OffsetDateTime::now_local().expect("valid time");
        let repr = date_time.format(&crate::FORMAT).expect("valid time");
        Self { date_time, repr }
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl std::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
