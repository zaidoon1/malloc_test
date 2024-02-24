use anyhow::{Context, Result};
use speedb::{self, LogLevel, Options};
use std::sync::Arc;

#[derive(Clone)]
pub struct DB {
    db: Arc<speedb::DBWithThreadMode<speedb::SingleThreaded>>,
}

impl DB {
    pub fn init<P>(path: P) -> Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        db_opts.enable_statistics();
        db_opts.set_log_level(LogLevel::Info);
        db_opts.set_dump_malloc_stats(true);

        let db = speedb::DBWithThreadMode::<speedb::SingleThreaded>::open(&db_opts, &path)
            .with_context(|| format!("failed to open DB. DB path: {}", path.as_ref().display()))?;

        let kv_store = DB { db: Arc::new(db) };

        Ok(kv_store)
    }
}
