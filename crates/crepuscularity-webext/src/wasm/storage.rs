use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::*;

use super::core::{self, BrowserError, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageAreaName {
    Local,
    Sync,
    Session,
    Managed,
}

impl StorageAreaName {
    pub fn as_str(self) -> &'static str {
        match self {
            StorageAreaName::Local => "local",
            StorageAreaName::Sync => "sync",
            StorageAreaName::Session => "session",
            StorageAreaName::Managed => "managed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageArea {
    name: StorageAreaName,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("storage")
}

pub fn local() -> StorageArea {
    StorageArea {
        name: StorageAreaName::Local,
    }
}

pub fn sync() -> StorageArea {
    StorageArea {
        name: StorageAreaName::Sync,
    }
}

pub fn session() -> StorageArea {
    StorageArea {
        name: StorageAreaName::Session,
    }
}

pub fn managed() -> StorageArea {
    StorageArea {
        name: StorageAreaName::Managed,
    }
}

pub async fn get<T>(key: &str) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    local().get_key(key).await
}

pub async fn set<T>(key: &str, value: &T) -> Result<()>
where
    T: Serialize,
{
    local().set_key(key, value).await
}

pub async fn remove(key: &str) -> Result<()> {
    local().remove(&key).await
}

impl StorageArea {
    pub fn name(self) -> StorageAreaName {
        self.name
    }

    pub async fn get_value<T>(&self, keys: &T) -> Result<JsValue>
    where
        T: Serialize,
    {
        let area = area_value(self.name)?;
        core::call_browser_method(
            &area,
            &format!("storage.{}.get", self.name.as_str()),
            "get",
            &[core::to_js(keys)?],
        )
        .await
    }

    pub async fn get<T, R>(&self, keys: &T) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        core::from_js(self.get_value(keys).await?)
    }

    pub async fn set<T>(&self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let area = area_value(self.name)?;
        core::call_browser_method(
            &area,
            &format!("storage.{}.set", self.name.as_str()),
            "set",
            &[core::to_js(value)?],
        )
        .await?;
        Ok(())
    }

    pub async fn remove<T>(&self, keys: &T) -> Result<()>
    where
        T: Serialize,
    {
        let area = area_value(self.name)?;
        core::call_browser_method(
            &area,
            &format!("storage.{}.remove", self.name.as_str()),
            "remove",
            &[core::to_js(keys)?],
        )
        .await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let area = area_value(self.name)?;
        core::call_browser_method(
            &area,
            &format!("storage.{}.clear", self.name.as_str()),
            "clear",
            &[],
        )
        .await?;
        Ok(())
    }

    pub async fn get_json(&self, keys: Value) -> Result<Value> {
        self.get(&keys).await
    }

    pub async fn get_key<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let values: Value = self.get(&key).await?;
        match values.get(key) {
            Some(value) if !value.is_null() => serde_json::from_value(value.clone())
                .map(Some)
                .map_err(|error| BrowserError::Serde(error.to_string())),
            _ => Ok(None),
        }
    }

    pub async fn set_key<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut values = serde_json::Map::new();
        values.insert(
            key.to_string(),
            serde_json::to_value(value).map_err(|error| BrowserError::Serde(error.to_string()))?,
        );
        self.set(&Value::Object(values)).await
    }
}

fn area_value(area: StorageAreaName) -> Result<JsValue> {
    core::get_path(&namespace()?, area.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local() {
        assert_eq!(local().name, StorageAreaName::Local);
    }

    #[test]
    fn test_sync() {
        assert_eq!(sync().name, StorageAreaName::Sync);
    }

    #[test]
    fn test_session() {
        assert_eq!(session().name, StorageAreaName::Session);
    }

    #[test]
    fn test_managed() {
        assert_eq!(managed().name, StorageAreaName::Managed);
    }
}
