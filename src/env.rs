use std::collections::HashMap;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Env {
    pub name: String,
    pub value: String,
    pub comment: Option<String>,
    pub expiration_date: Option<NaiveDate>,
}

impl Env {
    pub fn new(
        name: String,
        value: String,
        comment: Option<String>,
        expiration_date: Option<NaiveDate>,
    ) -> Env {
        Env {
            name,
            value,
            comment,
            expiration_date,
        }
    }

    pub fn from_key_value(key: String, value: String) -> Env {
        Env {
            name: key,
            value,
            comment: None,
            expiration_date: None,
        }
    }
}

/// Wrapper around a vector of `Env`
#[derive(Serialize, Deserialize, Clone)]
pub struct EnvVec {
    envs: Vec<Env>,
}

impl From<Vec<Env>> for EnvVec {
    fn from(envs: Vec<Env>) -> Self {
        EnvVec { envs }
    }
}

impl From<HashMap<String, String>> for EnvVec {
    fn from(envs: HashMap<String, String>) -> Self {
        let mut env_vec = EnvVec::new();

        for (key, value) in envs {
            env_vec.push(Env::from_key_value(key, value));
        }

        env_vec
    }
}

impl From<EnvVec> for Vec<Env> {
    fn from(val: EnvVec) -> Self {
        val.envs
    }
}

impl From<EnvVec> for HashMap<String, String> {
    fn from(val: EnvVec) -> Self {
        let mut envs = HashMap::new();

        for e in val.envs {
            envs.insert(e.name, e.value);
        }

        envs
    }
}

impl Default for EnvVec {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvVec {
    pub fn new() -> EnvVec {
        EnvVec { envs: Vec::new() }
    }

    pub fn push(&mut self, env: Env) {
        self.envs.push(env);
    }

    pub fn remove(&mut self, env: &str) {
        self.envs.retain(|e| e.name != env);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Env> {
        self.envs.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Env> {
        self.envs.iter_mut()
    }

    pub fn keys(&self) -> Vec<String> {
        self.envs.iter().map(|e| e.name.clone()).collect()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.envs.iter().any(|e| e.name == key)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        for e in self.envs.iter() {
            if e.name == key {
                return Some(&e.value);
            }
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.envs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.envs.len()
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Env) -> bool,
    {
        self.envs.retain(f);
    }
}

impl IntoIterator for EnvVec {
    type Item = Env;
    type IntoIter = std::vec::IntoIter<Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.into_iter()
    }
}

impl<'a> IntoIterator for &'a EnvVec {
    type Item = &'a Env;
    type IntoIter = std::slice::Iter<'a, Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.iter()
    }
}

impl<'a> IntoIterator for &'a mut EnvVec {
    type Item = &'a mut Env;
    type IntoIter = std::slice::IterMut<'a, Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.iter_mut()
    }
}

