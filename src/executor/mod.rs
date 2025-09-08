use std::sync::Arc;

use crate::{
    executor::catalog::CatalogRef,
    sql::{Statement, ast::Value},
    storage::{buffer_pool::BufferPoolManager, disk::DiskManager},
};
use async_trait::async_trait;
use bincode::{Decode, Encode};

pub mod catalog;
pub mod executors;

#[derive(Debug, Encode, Decode)]
pub struct Tuple {
    values: Vec<Value>,
}

#[derive(Debug, Encode, Decode)]
pub enum ExecutionResult {
    Message(String),
    Data(Vec<Tuple>),
}

#[async_trait(?Send)]
pub trait Executor {
    async fn execute(self: Box<Self>) -> Result<ExecutionResult, String>;
}

pub fn create_executor(
    stat: Statement,
    bpm: Arc<BufferPoolManager>,
    catalog: CatalogRef,
    disk_manager: Arc<DiskManager>,
) -> Box<dyn Executor> {
    match stat {
        Statement::CreateTable {
            table_name,
            columns,
        } => Box::new(executors::CreateTableExecutor {
            table_name,
            columns,
            catalog,
        }),
        Statement::Insert { table_name, values } => Box::new(executors::InsertExecutor {
            table_name,
            values,
            catalog,
            bpm,
            disk_manager,
        }),
        Statement::Select {
            table_name,
            columns,
        } => Box::new(executors::SequentialScanExecutor {
            table_name,
            columns,
            catalog,
            bpm,
            disk_manager,
        }),
    }
}
