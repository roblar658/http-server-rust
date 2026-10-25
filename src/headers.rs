use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeaderMap {
    // Nøkkel er lowercase for case-insensitiv oppslag, verdi er (OriginalKey, Value)
    headers: BTreeMap<String, (String, String)>,
}

impl HeaderMap {
    pub const CONTENT_TYPE: &'static str = "Content-Type";
    pub const CONTENT_LENGTH: &'static str = "Content-Length";
    pub const HOST: &'static str = "Host";
    pub const CONNECTION: &'static str = "Connection";
    pub const USER_AGENT: &'static str = "User-Agent";
    pub const DATE: &'static str = "Date";
    pub const SERVER: &'static str = "Server";

    pub fn new() -> Self {
        Self {
            headers: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let k = key.into();
        let v = value.into();
        self.headers.insert(k.to_ascii_lowercase(), (k, v));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(&key.to_ascii_lowercase()).map(|(_, v)| v.as_str())
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.headers.remove(&key.to_ascii_lowercase()).map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.headers.contains_key(&key.to_ascii_lowercase())
    }

    pub fn content_length(&self) -> Option<usize> {
        self.get(Self::CONTENT_LENGTH)
            .and_then(|val| val.trim().parse::<usize>().ok())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.values().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}

impl fmt::Display for HeaderMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, val) in self.iter() {
            write!(f, "{key}: {val}\r\n")?;
        }
        Ok(())
    }
}
