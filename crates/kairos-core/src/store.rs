use crate::episode::Episode;
use crate::{KairosError, Result};
use cozo::{DbInstance, ScriptMutability};

pub struct KairosStore {
    db: DbInstance,
}

impl KairosStore {
    pub fn memory() -> Result<Self> {
        let db = DbInstance::new("mem", "", "").map_err(|e| KairosError::Store(e.to_string()))?;
        let store = Self { db };
        store.init()?;
        Ok(store)
    }

    pub fn insert_episode(&self, episode: &Episode) -> Result<()> {
        let payload =
            serde_json::to_string(episode).map_err(|e| KairosError::Store(e.to_string()))?;
        let script = format!(
            "?[id, title, payload] <- [[{:?}, {:?}, {:?}]] :put episodes {{id => title, payload}}",
            episode.id, episode.title, payload
        );
        self.db
            .run_script(&script, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| KairosError::Store(e.to_string()))?;
        Ok(())
    }

    fn init(&self) -> Result<()> {
        self.db
            .run_script(
                ":create episodes {id => title, payload}",
                Default::default(),
                ScriptMutability::Mutable,
            )
            .map(|_| ())
            .map_err(|e| KairosError::Store(e.to_string()))
    }
}
