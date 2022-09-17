use serde::Serialize;
use uuid::Uuid;

use crate::{fetch::ImageKind, FetchImage};

#[derive(Debug)]
pub struct Image<T = ()>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    pub id: Uuid,
    pub url: String,
    pub kind: ImageKind,
    pub meta: T,
}

impl FetchImage for Image {
    fn url(&self) -> &str {
        &self.url
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn kind(&self) -> ImageKind {
        self.kind
    }
}

pub struct StoredImage<T>
where
    T: serde::Serialize,
    for<'de> T: serde::Deserialize<'de>,
{
    pub image: Image<T>,
    pub data: Box<[u8]>,
}

// XXX is this F needed?
// TODO why is this generic?
// BUG this breaks the type erasure
pub struct ImageStore<F> {
    conn: rusqlite::Connection,
    _marker: std::marker::PhantomData<F>,
}

impl<F> ImageStore<F>
where
    F: FetchImage,
{
    // TODO get this at runtime
    const DB_NAME: &'static str = "images.db";

    const SCHEMA: &'static str = r#"
        CREATE TABLE IF NOT EXISTS images (
            url    STRING NOT NULL UNIQUE,
            uuid   BLOB NOT NULL UNIQUE,
            kind   INTEGER NOT NULL,
            meta   STRING,
            data   BLOB NOT NULL
        );
        "#;

    fn ensure_schema(self) -> Self {
        self.conn.execute(Self::SCHEMA, []).expect("create table");
        self
    }

    fn open() -> Self {
        let conn = rusqlite::Connection::open(Self::DB_NAME).expect("open connection");
        conn.execute(Self::SCHEMA, []).expect("create table");
        Self {
            conn,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add(image: &F, meta: &impl Serialize, blob: &[u8]) -> bool {
        let this = Self::open();
        // TODO return an error
        let res = this.conn.execute(
            r#"
                INSERT INTO images (uuid, url, data, kind, meta)
                    VALUES (:uuid, :url, :data, :kind, :meta);
                "#,
            rusqlite::named_params! {
                ":uuid": image.id(),
                ":url": image.url(),
                ":kind": image.kind().to_tag(),
                ":meta": serde_json::to_string(meta).expect("valid json"),
                ":data": blob,
            },
        );

        matches!(res, Ok(1))
    }

    pub fn get_all_debug() -> Vec<StoredImage<String>> {
        let conn = Self::open().conn;
        let mut stmt = conn
            .prepare("SELECT uuid, url, data, kind, meta FROM images")
            .expect("valid sql");
        let iter = match stmt.query_map([], Self::from_row_erased) {
            Ok(iter) => iter,
            Err(_) => return vec![],
        };

        iter.into_iter().flatten().collect()
    }

    pub fn get_id(url: &str) -> Option<Uuid> {
        let conn = Self::open().conn;
        let mut stmt = conn
            .prepare("SELECT uuid FROM images WHERE url = :url LIMIT 1;")
            .expect("valid sql");

        let iter = stmt.query_map(
            rusqlite::named_params! {
                ":url": url
            },
            |row| row.get("uuid"),
        );

        let mut row = iter.ok()?;
        row.next().transpose().ok().flatten()
    }

    pub fn has_id(id: Uuid) -> bool {
        pub fn inner<F>(ImageStore { conn, .. }: ImageStore<F>, id: Uuid) -> rusqlite::Result<u64>
        where
            F: FetchImage,
        {
            let mut stmt =
                conn.prepare("SELECT COUNT(uuid) as [count] FROM images WHERE uuid = :uuid;")?;
            let mut rows = stmt.query(rusqlite::named_params! {
                ":uuid": id,
            })?;
            let row = rows.next()?;
            Ok(row.and_then(|row| row.get_unwrap(0)).unwrap_or_default())
        }

        matches!(inner(Self::open(), id), Ok(1))
    }

    pub fn get<T>(id: Uuid) -> Option<StoredImage<T>>
    where
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + Default,
    {
        let res = Self::open().conn.query_row(
            "SELECT uuid, url, meta, kind, data FROM images WHERE uuid = :uuid;",
            rusqlite::named_params! {
                ":uuid": id,
            },
            Self::from_row,
        );

        res.ok()
    }

    pub fn remove(id: Uuid) -> bool {
        let res = Self::open().conn.execute(
            "DELETE FROM images WHERE uuid = :uuid;",
            rusqlite::named_params! {
                ":uuid": id,
            },
        );

        matches!(res, Ok(1))
    }

    pub fn remove_url(url: &str) -> bool {
        let res = Self::open().conn.execute(
            "DELETE FROM images WHERE url = :url;",
            rusqlite::named_params! {
                ":url": url,
            },
        );

        matches!(res, Ok(1))
    }

    fn from_row_erased(row: &rusqlite::Row) -> rusqlite::Result<StoredImage<String>> {
        Self::from_row(row)
    }

    fn from_row<T>(row: &rusqlite::Row) -> rusqlite::Result<StoredImage<T>>
    where
        T: serde::Serialize + for<'de> serde::Deserialize<'de>,
        T: Default,
    {
        let id = row.get_unwrap("uuid");
        let url = row.get_unwrap("url");
        let data = row.get_unwrap::<_, Vec<u8>>("data").into_boxed_slice();
        let kind = ImageKind::from_tag(row.get_unwrap("kind")) //
            .ok_or(rusqlite::Error::InvalidQuery)?;

        let meta = row
            .get_unwrap::<_, Option<String>>("meta")
            .filter(|s| !s.is_empty() && s != "null")
            .map(|s| serde_json::from_str(&dbg!(s)).expect("valid json"))
            .unwrap_or_default();

        let image = Image {
            id,
            url,
            kind,
            meta,
        };

        Ok(StoredImage { image, data })
    }
}
