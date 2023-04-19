use std::{collections::HashMap, fmt::Debug, ops::Deref, str::FromStr, sync::Arc};

#[derive(Clone)]
pub struct Env(Arc<EnvVars>);

impl Env {
    pub fn load() -> Self {
        let raw = std::fs::read_to_string(".env").ok();
        let map = raw.as_ref().map(|raw| {
            raw.lines()
                .filter(|l| !l.starts_with('#'))
                .filter_map(|l| l.split_once('='))
                .collect()
        });

        Self(Arc::new(EnvVars::load(map)))
    }
}

impl Deref for Env {
    type Target = Arc<EnvVars>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct EnvVars {
    pub database_url: String,
    pub env: String,
}

impl EnvVars {
    fn load(mut map: Option<HashMap<&str, &str>>) -> Self {
        let vars = Self {
            database_url: read(&mut map, "DATABASE_URL"),
            env: read(&mut map, "ENV"),
        };

        if let Some(m) = map.filter(|m| !m.is_empty()) {
            panic!("Env file has extra keys: {:?}", m.keys());
        }

        vars
    }
}

fn read<Parsed, ParseError>(map: &mut Option<HashMap<&str, &str>>, key: &str) -> Parsed
where
    ParseError: Debug,
    Parsed: FromStr<Err = ParseError>,
{
    if let Some(map) = map.as_mut() {
        if let Some(val) = map.remove(key).map(ToOwned::to_owned) {
            std::env::set_var(key, val.replace("\\n", "\n"));
        }
    }

    std::env::var(key)
        .unwrap_or_else(|_| panic!("No {key} environment variable found"))
        .parse()
        .unwrap_or_else(|_| panic!("Could not parse {key} environment variable"))
}
