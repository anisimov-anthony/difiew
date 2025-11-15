pub mod command;
pub mod error;
pub mod result;
use command::*;
use error::*;
use monotree::*;
use result::*;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::HashMap;
use std::result::Result as StdResult;

pub struct Store {
    root: Option<Hash>,
    monotree: Monotree<DefaultDatabase, DefaultHasher>,
    main_store: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            monotree: Monotree::default(),
            root: None,
            main_store: HashMap::new(),
        }
    }

    pub fn execute(&mut self, cmd: StoreCommand) -> StdResult<StoreCommandResult<'_>, StoreError> {
        match cmd {
            StoreCommand::DEL(DELParams { keys }) => {
                let count = self.del(&keys)?;
                Ok(StoreCommandResult::del(count))
            }
            StoreCommand::EXISTS(EXISTSParams { keys }) => {
                let count = self.exists(&keys);
                Ok(StoreCommandResult::exists(count))
            }
            StoreCommand::GET(GETParams { key }) => {
                let value = self.get(&key);
                Ok(StoreCommandResult::get(value))
            }
            StoreCommand::KEYS(KEYSParams { pattern }) => {
                let keys = self.keys(&pattern)?;
                Ok(StoreCommandResult::keys(keys))
            }
            StoreCommand::SET(SETParams { key, value }) => {
                let is_ok = self.set(&key, &value)?;
                Ok(StoreCommandResult::set(is_ok))
            }
        }
    }

    fn del(&mut self, keys: &[Cow<'_, str>]) -> StdResult<usize, StoreError> {
        let mut removed = 0;
        for key in keys {
            if self.main_store.remove(key.as_ref()).is_some() {
                let key_hash: [u8; 32] = Sha256::digest(key.as_bytes()).into();
                self.root = self
                    .monotree
                    .remove(self.root.as_ref(), &key_hash)
                    .map_err(StoreError::from)?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn exists(&self, keys: &[Cow<'_, str>]) -> usize {
        keys.iter()
            .filter(|k| self.main_store.contains_key(k.as_ref()))
            .count()
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.main_store.get(key).map(|s| s.as_str())
    }

    fn keys(&self, pattern: &str) -> StdResult<Vec<&str>, StoreError> {
        if pattern == "*" {
            return Ok(self.main_store.keys().map(|k| k.as_str()).collect());
        }

        let regex_pattern = pattern.replace("*", ".*");
        let re = regex::Regex::new(&format!("^{regex_pattern}$")).map_err(StoreError::from)?;

        Ok(self
            .main_store
            .keys()
            .filter(|k| re.is_match(k))
            .map(|k| k.as_str())
            .collect())
    }

    fn set(&mut self, key: &str, value: &str) -> StdResult<bool, StoreError> {
        let key_hash: [u8; 32] = Sha256::digest(key.as_bytes()).into();
        let value_hash: [u8; 32] = Sha256::digest(value.as_bytes()).into();

        self.main_store.insert(key.to_string(), value.to_string());
        self.root = self
            .monotree
            .insert(self.root.as_ref(), &key_hash, &value_hash)
            .map_err(StoreError::from)?;

        Ok(true)
    }

    pub fn reveal_root(&self) -> Option<Hash> {
        self.root
    }

    pub fn get_main_store(&self) -> HashMap<String, String> {
        self.main_store.clone()
    }

    pub fn update_full_store(
        &mut self,
        main_store: HashMap<String, String>,
    ) -> std::result::Result<(), StoreError> {
        self.main_store = main_store.clone();
        self.monotree = Monotree::default();
        self.root = None;

        for (key, value) in main_store.iter() {
            self.set(key, value)?;
        }
        Ok(())
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::result::Result as StdResult;

    fn set_keys(store: &mut Store, pairs: &[(&str, &str)]) -> StdResult<(), StoreError> {
        for &(k, v) in pairs {
            store.execute(StoreCommand::SET(SETParams {
                key: Cow::Borrowed(k),
                value: Cow::Borrowed(v),
            }))?;
        }
        Ok(())
    }

    #[test]
    fn test_set_and_get_basic() -> StdResult<(), StoreError> {
        let mut store = Store::new();

        // SET
        let result = store.execute(StoreCommand::SET(SETParams {
            key: Cow::Borrowed("view"),
            value: Cow::Borrowed("different"),
        }))?;
        assert_eq!(result, StoreCommandResult::set(true));

        // GET existing
        let result = store.execute(StoreCommand::GET(GETParams {
            key: Cow::Borrowed("view"),
        }))?;
        assert_eq!(result, StoreCommandResult::get(Some("different")));

        // GET non-existent
        let result = store.execute(StoreCommand::GET(GETParams {
            key: Cow::Borrowed("nope"),
        }))?;
        assert_eq!(result, StoreCommandResult::get::<&str>(None));

        Ok(())
    }

    #[test]
    fn test_set_checking_for_overwriting() -> StdResult<(), StoreError> {
        let mut store = Store::new();

        store.execute(StoreCommand::SET(SETParams {
            key: Cow::Borrowed("view"),
            value: Cow::Borrowed("different"),
        }))?;

        let result = store.execute(StoreCommand::GET(GETParams {
            key: Cow::Borrowed("view"),
        }))?;
        assert_eq!(result, StoreCommandResult::get(Some("different")));

        store.execute(StoreCommand::SET(SETParams {
            key: Cow::Borrowed("view"),
            value: Cow::Borrowed("another"),
        }))?;

        let result = store.execute(StoreCommand::GET(GETParams {
            key: Cow::Borrowed("view"),
        }))?;
        assert_eq!(result, StoreCommandResult::get(Some("another")));

        Ok(())
    }

    #[test]
    fn test_exists_exact_match() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
                ("fourth", "some_data"),
                ("fifth", "some_data"),
            ],
        )?;

        let result = store.execute(StoreCommand::EXISTS(EXISTSParams {
            keys: vec![
                Cow::Borrowed("first"),
                Cow::Borrowed("second"),
                Cow::Borrowed("third"),
                Cow::Borrowed("fourth"),
                Cow::Borrowed("fifth"),
            ],
        }))?;

        assert_eq!(result, StoreCommandResult::exists(5));
        Ok(())
    }

    #[test]
    fn test_exists_partial_match() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
                ("fourth", "some_data"),
                ("fifth", "some_data"),
            ],
        )?;

        let result = store.execute(StoreCommand::EXISTS(EXISTSParams {
            keys: vec![
                Cow::Borrowed("first"),
                Cow::Borrowed("second"),
                Cow::Borrowed("third"),
            ],
        }))?;

        assert_eq!(result, StoreCommandResult::exists(3));
        Ok(())
    }

    #[test]
    fn test_exists_redudant_match() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
            ],
        )?;

        let result = store.execute(StoreCommand::EXISTS(EXISTSParams {
            keys: vec![
                Cow::Borrowed("first"),
                Cow::Borrowed("second"),
                Cow::Borrowed("third"),
                Cow::Borrowed("fourth"),
                Cow::Borrowed("fifth"),
            ],
        }))?;

        assert_eq!(result, StoreCommandResult::exists(3));
        assert!(store.get("fourth").is_none());
        assert!(store.get("fifth").is_none());
        Ok(())
    }

    #[test]
    fn test_del_exact_match() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
                ("fourth", "some_data"),
                ("fifth", "some_data"),
            ],
        )?;

        let result = store.execute(StoreCommand::DEL(DELParams {
            keys: vec![
                Cow::Borrowed("first"),
                Cow::Borrowed("second"),
                Cow::Borrowed("third"),
                Cow::Borrowed("fourth"),
                Cow::Borrowed("fifth"),
            ],
        }))?;

        assert_eq!(result, StoreCommandResult::del(5));
        assert!(store.get("first").is_none());
        assert!(store.get("second").is_none());
        assert!(store.get("third").is_none());
        assert!(store.get("fourth").is_none());
        assert!(store.get("fifth").is_none());
        Ok(())
    }

    #[test]
    fn test_del_partial_match() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
                ("fourth", "some_data"),
                ("fifth", "some_data"),
            ],
        )?;

        let result = store.execute(StoreCommand::DEL(DELParams {
            keys: vec![
                Cow::Borrowed("first"),
                Cow::Borrowed("second"),
                Cow::Borrowed("third"),
            ],
        }))?;

        assert_eq!(result, StoreCommandResult::del(3));
        assert!(store.get("first").is_none());
        assert!(store.get("second").is_none());
        assert!(store.get("third").is_none());
        assert!(store.get("fourth").is_some());
        assert!(store.get("fifth").is_some());
        Ok(())
    }

    #[test]
    fn test_del_redudant_match() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
            ],
        )?;

        let result = store.execute(StoreCommand::DEL(DELParams {
            keys: vec![
                Cow::Borrowed("first"),
                Cow::Borrowed("second"),
                Cow::Borrowed("third"),
                Cow::Borrowed("fourth"),
                Cow::Borrowed("fifth"),
            ],
        }))?;

        assert_eq!(result, StoreCommandResult::del(3));
        assert!(store.get("first").is_none());
        assert!(store.get("second").is_none());
        assert!(store.get("third").is_none());
        assert!(store.get("fourth").is_none());
        assert!(store.get("fifth").is_none());
        Ok(())
    }

    #[test]
    fn test_keys_empty_store() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        let result = store.execute(StoreCommand::KEYS(KEYSParams {
            pattern: Cow::Borrowed("*"),
        }))?;

        let keys = match result {
            StoreCommandResult::KEYS(keys) => keys,
            _ => panic!("Expected KEYS variant"),
        };
        assert_eq!(keys.payload.len(), 0);
        Ok(())
    }

    #[test]
    fn test_keys_wildcard_star() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("user:User1", "some_data"),
                ("user:User2", "some_data"),
                ("admin:Admin1", "some_data"),
                ("temp:tmp1", "some_data"),
            ],
        )?;

        let result = store.execute(StoreCommand::KEYS(KEYSParams {
            pattern: Cow::Borrowed("*"),
        }))?;

        let keys = match result {
            StoreCommandResult::KEYS(keys) => keys,
            _ => panic!("Expected KEYS variant"),
        };
        assert_eq!(keys.payload.len(), 4);
        assert!(keys.payload.contains(&Cow::Borrowed("user:User1")));
        assert!(keys.payload.contains(&Cow::Borrowed("user:User2")));
        assert!(keys.payload.contains(&Cow::Borrowed("admin:Admin1")));
        assert!(keys.payload.contains(&Cow::Borrowed("temp:tmp1")));
        Ok(())
    }

    #[test]
    fn test_keys_prefix() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("user:User1", "some_data"),
                ("user:User2", "some_data"),
                ("admin:Admin1", "some_data"),
            ],
        )?;

        {
            let result = store.execute(StoreCommand::KEYS(KEYSParams {
                pattern: Cow::Borrowed("user:*"),
            }))?;
            let keys = match result {
                StoreCommandResult::KEYS(keys) => keys,
                _ => panic!("Expected KEYS variant"),
            };
            assert_eq!(keys.payload.len(), 2);
            assert!(keys.payload.contains(&Cow::Borrowed("user:User1")));
            assert!(keys.payload.contains(&Cow::Borrowed("user:User2")));
        }

        {
            let result = store.execute(StoreCommand::KEYS(KEYSParams {
                pattern: Cow::Borrowed("admin:*"),
            }))?;
            let keys = match result {
                StoreCommandResult::KEYS(keys) => keys,
                _ => panic!("Expected KEYS variant"),
            };
            assert_eq!(keys.payload.len(), 1);
            assert!(keys.payload.contains(&Cow::Borrowed("admin:Admin1")));
        }
        Ok(())
    }

    #[test]
    fn test_keys_exact_match() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
            ],
        )?;

        for &expected in &["first", "second", "third"] {
            let result = store.execute(StoreCommand::KEYS(KEYSParams {
                pattern: Cow::Borrowed(expected),
            }))?;
            let keys = match result {
                StoreCommandResult::KEYS(keys) => keys,
                _ => panic!("Expected KEYS variant"),
            };
            assert_eq!(keys.payload, vec![expected]);
        }
        Ok(())
    }

    #[test]
    fn test_monotree_root_updates_on_set() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        let root_before = store.reveal_root();

        store.execute(StoreCommand::SET(SETParams {
            key: Cow::Borrowed("view"),
            value: Cow::Borrowed("different"),
        }))?;

        let root_after = store.reveal_root();
        assert_ne!(root_before, root_after);
        assert!(root_after.is_some());
        Ok(())
    }

    #[test]
    fn test_monotree_root_updates_on_del() -> StdResult<(), StoreError> {
        let mut store = Store::new();

        store.execute(StoreCommand::SET(SETParams {
            key: Cow::Borrowed("view"),
            value: Cow::Borrowed("different"),
        }))?;
        let root_after_set = store.reveal_root();

        store.execute(StoreCommand::DEL(DELParams {
            keys: vec![Cow::Borrowed("view")],
        }))?;

        let root_after_del = store.reveal_root();
        assert_ne!(root_after_set, root_after_del);
        Ok(())
    }

    #[test]
    fn test_update_full_store_resets_and_rebuilds() -> StdResult<(), StoreError> {
        let mut store = Store::new();
        set_keys(
            &mut store,
            &[
                ("first", "some_data"),
                ("second", "some_data"),
                ("third", "some_data"),
            ],
        )?;

        let old_root = store.reveal_root();

        let new_data = HashMap::from([
            ("fourth".to_string(), "some_data".to_string()),
            ("fifth".to_string(), "some_data".to_string()),
        ]);

        store.update_full_store(new_data.clone())?;

        assert_eq!(store.get_main_store(), new_data);
        assert_eq!(store.get("fourth"), Some("some_data"));
        assert_eq!(store.get("sixth"), None);

        let new_root = store.reveal_root();
        assert_ne!(old_root, new_root);
        Ok(())
    }
}
