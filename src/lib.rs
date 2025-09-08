use std::sync::{Arc, Mutex};

use crate::{
    executor::{
        ExecutionResult,
        catalog::{Catalog, CatalogRef},
        create_executor,
    },
    sql::parse_sql,
    storage::{buffer_pool::BufferPoolManager, disk::DiskManager},
};

pub mod executor;
pub mod sql;
pub mod storage;

pub struct Database {
    bpm: Arc<BufferPoolManager>,
    catalog: CatalogRef,
}

impl Database {
    pub async fn new(db_file: String, pool_size: usize) -> Result<Self, String> {
        let _disk_manager = Arc::new(
            DiskManager::new(&db_file)
                .await
                .map_err(|e| e.to_string())?,
        );
        let bpm = BufferPoolManager::new(pool_size);
        let catalog = Arc::new(Mutex::new(Catalog::default()));
        Ok(Self {
            bpm,
            catalog,
        })
    }

    // This function is now async
    pub async fn run_statement(&self, sql: &str, disk_manager: Arc<DiskManager>) -> Result<ExecutionResult, String> {
        let ast = parse_sql(sql).map_err(|e| e.to_string())?;
        let executor = create_executor(ast, self.bpm.clone(), self.catalog.clone(), disk_manager);
        // Await the executor's result
        executor.execute().await
    }
}
