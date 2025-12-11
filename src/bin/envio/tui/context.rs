use envio::Profile;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAX_AGE: Duration = Duration::from_secs(10); // 10 seconds

#[derive(Clone)]
struct Cached<T> {
    value: T,
    timestamp: Instant,
}

impl<T> Cached<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() >= MAX_AGE
    }
}

#[derive(Clone)]
pub struct ProfileCache {
    profiles: HashMap<String, Cached<Profile>>,
}

impl ProfileCache {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    pub fn has_profile(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }

    pub fn get_profile(&mut self, name: &str) -> Option<Profile> {
        if let Some(cached) = self.profiles.get(name)
            && !cached.is_expired()
        {
            return Some(cached.value.clone());
        }

        self.profiles.remove(name);
        None
    }

    pub fn insert_profile(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, Cached::new(profile));
    }

    pub fn cleanup_expired(&mut self) {
        self.profiles.retain(|_, v| !v.is_expired());
    }
}

pub struct AppContext {
    pub cache: ProfileCache,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            cache: ProfileCache::new(),
        }
    }
}
