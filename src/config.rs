use anyhow::Context;

#[derive(Clone)]
pub struct EnvConfig {
    pub twitch_name: String,
    pub twitch_oauth_token: String,

    pub twitch_client_id: String,
    pub twitch_client_secret: String,
}

impl EnvConfig {
    pub fn load_from_env() -> anyhow::Result<Self> {
        fn get_env(key: &str) -> anyhow::Result<String> {
            std::env::var(key).with_context(|| anyhow::anyhow!("expected `{key}` in the env"))
        }

        Ok(Self {
            twitch_name: get_env("TWITCH_NAME")?,
            twitch_oauth_token: get_env("TWITCH_OAUTH_TOKEN")?,

            twitch_client_id: get_env("TWITCH_CLIENT_ID")?,
            twitch_client_secret: get_env("TWITCH_CLIENT_secret")?,
        })
    }
}
