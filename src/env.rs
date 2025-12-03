use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Serialize, Deserialize, Clone)]
pub struct Env {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
    pub expiration_date: Option<NaiveDate>,
}

impl Env {
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        comment: Option<String>,
        expiration_date: Option<NaiveDate>,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            comment,
            expiration_date,
        }
    }

    pub fn from_key_value(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(key, value, None, None)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EnvMap {
    envs: IndexMap<String, Env>,
}

impl Default for EnvMap {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvMap {
    pub fn new() -> Self {
        Self {
            envs: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, env: Env) {
        self.envs.insert(env.key.clone(), env);
    }

    pub fn insert_from_key_value(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.insert(Env::from_key_value(key, value));
    }

    pub fn remove(&mut self, key: &str) -> Result<()> {
        if self.envs.shift_remove(key).is_none() {
            return Err(Error::EnvDoesNotExist(key.to_string()));
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Env> {
        self.envs.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.envs.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.envs.keys()
    }

    pub fn len(&self) -> usize {
        self.envs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.envs.is_empty()
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Env) -> bool,
    {
        self.envs.retain(|_, env| f(env));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Env> {
        self.envs.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Env> {
        self.envs.values_mut()
    }
}

impl From<IndexMap<String, String>> for EnvMap {
    fn from(map: IndexMap<String, String>) -> Self {
        let mut env_map = Self::new();
        for (k, v) in map {
            env_map.insert(Env::from_key_value(k, v));
        }
        env_map
    }
}

impl From<EnvMap> for IndexMap<String, String> {
    fn from(env_map: EnvMap) -> IndexMap<String, String> {
        env_map
            .envs
            .into_iter()
            .map(|(k, v)| (k, v.value))
            .collect()
    }
}

impl From<Vec<Env>> for EnvMap {
    fn from(envs: Vec<Env>) -> Self {
        let mut env_map = Self::new();

        for env in envs {
            env_map.insert(env);
        }

        env_map
    }
}

impl IntoIterator for EnvMap {
    type Item = Env;
    type IntoIter = indexmap::map::IntoValues<String, Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.into_values()
    }
}

impl<'a> IntoIterator for &'a EnvMap {
    type Item = &'a Env;
    type IntoIter = indexmap::map::Values<'a, String, Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.values()
    }
}

impl<'a> IntoIterator for &'a mut EnvMap {
    type Item = &'a mut Env;
    type IntoIter = indexmap::map::ValuesMut<'a, String, Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.values_mut()
    }
}
