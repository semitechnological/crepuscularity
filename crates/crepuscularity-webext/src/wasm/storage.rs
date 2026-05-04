use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::*;

use super::core::{self, Result};

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

impl StorageArea {
    pub fn name(self) -> StorageAreaName {
        self.name
    }

    pub async fn get_value<T>(&self, keys: &T) -> Result<JsValue>
    where
        T: Serialize,
    {
        let area = area_value(self.name)?;
        core::call_method(
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
        core::call_method(
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
        core::call_method(
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
        core::call_method(
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
}

fn area_value(area: StorageAreaName) -> Result<JsValue> {
    core::get_path(&namespace()?, area.as_str())
}
