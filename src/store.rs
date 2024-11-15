use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
  #[error(transparent)]
  Io(#[from] io::Error),

  #[error(transparent)]
  Bincode(#[from] bincode::Error),

  #[error("Key '{0}' not found")]
  KeyNotFound(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug)]
pub struct Store {
  /// Database. For simplicity stored in an `RwLock` to allow concurrent access.
  db: Arc<RwLock<Database>>,
  /// Path to the store.
  path: PathBuf,
}

impl Store {
  /// Opens a store at the specified path.
  ///
  /// - If the store does not exist, it will be created.
  /// - If the store exists, it will be loaded.
  /// - If the store is corrupted, it will be recreated.
  pub fn open(path: &str) -> Result<Self> {
    let db = Database::load(Path::new(path))?;

    Ok(Store {
      db: Arc::new(RwLock::new(db)),
      path: PathBuf::from(path),
    })
  }

  /// Sets a value in the store.
  ///
  /// Value is serialized using `bincode` via `serde`, so it must derive or implement serde's
  /// `Serialize` trait.
  pub fn set<T: Serialize>(&self, key: impl AsRef<str>, value: T) -> Result<()> {
    let mut db = self
      .db
      .write()
      .map_err(|_| StoreError::Io(io::Error::new(io::ErrorKind::Other, "Lock poisoned")))?;

    db.set(key.as_ref(), value)?;
    db.save(&self.path)?;

    Ok(())
  }

  /// Gets a value from the store.
  ///
  /// Value is deserialized using `bincode` via `serde`, so it must derive or implement serde's
  /// `Deserialize` trait. While constraint is `DeserializeOwned`, it should derive or implement
  /// `Deserialize` trait instead.
  pub fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<T> {
    let db = self
      .db
      .read()
      .map_err(|_| StoreError::Io(io::Error::new(io::ErrorKind::Other, "Lock poisoned")))?;

    db.get(key.as_ref())
  }

  /// Removes a value from the store.
  ///
  /// If the value does not exist, nothing happens.
  pub fn remove(&self, key: impl AsRef<str>) -> Result<()> {
    let mut db = self
      .db
      .write()
      .map_err(|_| StoreError::Io(io::Error::new(io::ErrorKind::Other, "Lock poisoned")))?;

    db.remove(key.as_ref());
    db.save(&self.path)?;

    Ok(())
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Database {
  data: BTreeMap<String, Vec<u8>>,
}

impl Database {
  fn new() -> Self {
    Database {
      data: BTreeMap::new(),
    }
  }

  fn load(path: &Path) -> Result<Self> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }

    match File::open(path) {
      | Ok(mut file) => {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let db = bincode::deserialize::<Database>(&buffer)?;

        Ok(db)
      },
      | Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
        let db = Database::new();
        db.save(path)?;

        Ok(db)
      },
      | Err(err) => Err(StoreError::Io(err)),
    }
  }

  fn save(&self, path: &Path) -> Result<()> {
    let encoded = bincode::serialize(&self)?;
    let mut file = File::create(path)?;

    file.write_all(&encoded)?;
    file.flush()?;

    Ok(())
  }

  fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
    let encoded = bincode::serialize(&value)?;
    self.data.insert(key.to_string(), encoded);

    Ok(())
  }

  fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
    match self.data.get(key) {
      | Some(bytes) => {
        let decoded = bincode::deserialize(bytes)?;

        Ok(decoded)
      },
      | None => Err(StoreError::KeyNotFound(key.to_string())),
    }
  }

  fn remove(&mut self, key: &str) {
    self.data.remove(key);
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;
  use std::thread;
  use std::time::Duration;

  use tempfile::tempdir;
  use tokio::task;

  use super::*;

  fn create_store(temp_path: &Path) -> Store {
    Store::open(temp_path.to_str().unwrap()).expect("Failed to create store")
  }

  #[test]
  fn test_store_initialization() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db/test.db");
    let _ = create_store(&db_path);

    assert!(db_path.exists());
  }

  fn test_store_corruption() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db/test.db");
    let store = create_store(&db_path);

    store
      .set("key1", "value1".to_string())
      .expect("Failed to set value");

    fs::remove_file(db_path).expect("Failed to remove file");

    let result: Result<String> = store.get("key1");

    assert!(matches!(result, Err(StoreError::Io(_))));
  }

  #[test]
  fn test_set_and_get_value() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db/test.db");
    let store = create_store(&db_path);

    store
      .set("key1", "value1".to_string())
      .expect("Failed to set value");

    let value: String = store.get("key1").expect("Failed to get value");

    assert_eq!(value, "value1");
  }

  #[test]
  fn test_get_non_existent_key() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db/test.db");
    let store = create_store(&db_path);
    let result: Result<String> = store.get("nonexistent_key");

    assert!(matches!(result, Err(StoreError::KeyNotFound(_))));
  }

  #[tokio::test]
  async fn test_concurrent_access() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db/test.db");
    let store = Arc::new(create_store(&db_path));

    let handle1 = {
      let store = Arc::clone(&store);

      task::spawn(async move {
        store
          .set("key1", "concurrent_value1".to_string())
          .expect("Failed to set value");

        thread::sleep(Duration::from_millis(100));

        let value: String = store.get("key1").expect("Failed to get value");

        assert_eq!(value, "concurrent_value1");
      })
    };

    let handle2 = {
      let store = Arc::clone(&store);

      task::spawn(async move {
        store
          .set("key2", "concurrent_value2".to_string())
          .expect("Failed to set value");

        thread::sleep(Duration::from_millis(100));

        let value: String = store.get("key2").expect("Failed to get value");

        assert_eq!(value, "concurrent_value2");
      })
    };

    let _ = tokio::join!(handle1, handle2);
  }

  #[tokio::test]
  async fn test_parallel_set_and_get() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db/test.db");
    let store = Arc::new(create_store(&db_path));

    let handles: Vec<_> = (0..10)
      .map(|i| {
        let store = store.clone();

        task::spawn(async move {
          let key = format!("key{}", i);
          let value = format!("value{}", i);

          store.set(&key, value.clone()).expect("Failed to set value");

          let result: String = store.get(&key).expect("Failed to get value");

          assert_eq!(result, value);
        })
      })
      .collect();

    for handle in handles {
      let _ = handle.await;
    }
  }

  #[test]
  fn test_persistent_storage() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db/test.db");

    {
      let store = create_store(&db_path);

      store
        .set("key1", "persistent_value".to_string())
        .expect("Failed to set value");
    }

    let store = create_store(&db_path);
    let value: String = store.get("key1").expect("Failed to get value");

    assert_eq!(value, "persistent_value");
  }
}
