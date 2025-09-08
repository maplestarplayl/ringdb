use crate::{
    executor::{catalog::CatalogRef, ExecutionResult, Executor, Tuple},
    sql::ast,
    storage::{
        buffer_pool::BufferPoolManager, disk::DiskManager, page::{Page, PAGE_SIZE}
    },
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct CreateTableExecutor {
    pub(crate) table_name: String,
    pub(crate) columns: Vec<ast::Column>,
    pub(crate) catalog: CatalogRef,
}

#[async_trait(?Send)]
impl Executor for CreateTableExecutor {
    async fn execute(self: Box<Self>) -> Result<ExecutionResult, String> {
        let mut catalog = self.catalog.lock().unwrap();
        if catalog
            .create_table(self.table_name.clone(), self.columns)
            .is_ok()
        {
            Ok(ExecutionResult::Message(format!(
                "Table '{}' created.",
                self.table_name
            )))
        } else {
            Err(format!("Table '{}' already exists.", self.table_name))
        }
    }
}

pub struct InsertExecutor {
    pub table_name: String,
    pub values: Vec<ast::Value>,
    pub catalog: CatalogRef,
    pub bpm: Arc<BufferPoolManager>,
    pub disk_manager: Arc<DiskManager>,
}

#[async_trait(?Send)]
impl Executor for InsertExecutor {
    async fn execute(self: Box<Self>) -> Result<ExecutionResult, String> {
        let table_info = {
            let catalog = self.catalog.lock().unwrap();
            catalog.get_table(&self.table_name).cloned()
        };

        if table_info.is_none() {
            return Err(format!("Table '{}' not found.", self.table_name));
        }

        let tuple = Tuple {
            values: self.values,
        };
        // 使用 bincode 的标准函数进行序列化
        let tuple_data = bincode::encode_to_vec(&tuple, bincode::config::standard())
            .map_err(|e| e.to_string())?;

        // 极简实现：总是尝试插入到第一页
        let page_id = 0;

        let mut page_write_guard = self
            .bpm
            .fetch_page_mut(page_id, self.disk_manager.clone())
            .await
            .map_err(|e| format!("Failed to fetch page {}: {:?}", page_id, e))?;
        let frame_id = page_write_guard.frame_id();
        let mut frames_mut = page_write_guard.get_mut();
        let mut page = Page::from_bytes(frames_mut[frame_id].data[..PAGE_SIZE].try_into().unwrap());

        if page.insert_tuple(&tuple_data).is_some() {
            // Write the modified page back into the frame
            frames_mut[frame_id].data.copy_from_slice(&page.to_bytes());
            Ok(ExecutionResult::Message("1 row inserted.".to_string()))
        } else {
            Err("Failed to insert tuple: page is full.".to_string())
        }
    }
}

pub struct SequentialScanExecutor {
    pub table_name: String,
    pub columns: Vec<String>,
    pub catalog: CatalogRef,
    pub bpm: Arc<BufferPoolManager>,
    pub disk_manager: Arc<DiskManager>,
}

#[async_trait(?Send)]
impl Executor for SequentialScanExecutor {
    async fn execute(mut self: Box<Self>) -> Result<ExecutionResult, String> {
        let _table_info = {
            let catalog = self.catalog.lock().unwrap();
            catalog.get_table(&self.table_name).cloned()
        }
        .ok_or_else(|| format!("Table '{}' not found", self.table_name))?;

        const PREFETCH_PAGES: usize = 16;

        const TOTAL_PAGES: usize = 128; //TODO: HArdcore for now


        // 1. Start the prefetching task
        let mut result_tuples: Vec<Tuple> = Vec::new();
        let (tx, mut rx) = local_sync::mpsc::bounded::channel(PREFETCH_PAGES);
        let bpm_clone = self.bpm.clone();

        let prefetch_handle = monoio::spawn(async move {
            let mut futures = Vec::new();
            for page_id in 0..TOTAL_PAGES {
                let fetch_fut = bpm_clone.fetch_page(page_id as _, self.disk_manager.clone());
                futures.push(fetch_fut);

                if futures.len() >= PREFETCH_PAGES {
                    let page_guards = futures::future::join_all(futures.drain(..)).await;
                    for guard_result in page_guards {
                        match guard_result {
                            Ok(guard) => {
                                if tx.send(guard).await.is_err() {
                                    return;
                                }
                            }
                            Err(_) => {
                                eprintln!("Failed to fetch page");
                                return;
                            }
                        }
                    }
                }
            }

            // 处理剩余的 futures
            if !futures.is_empty() {
                 let page_guards = futures::future::join_all(futures.drain(..)).await;
                 for guard_result in page_guards {
                     match guard_result {
                         Ok(guard) => {
                             if tx.send(guard).await.is_err() { return; }
                         },
                         Err(_) => { return; }
                     }
                 }
            }
        });


        // 2. Consume the prefetched pages
        while let Some(page_guard) = rx.recv().await {
            let frame_id = page_guard.frame_id();
            let frames = page_guard.get();
            let page = Page::from_bytes(frames[frame_id].data[..PAGE_SIZE].try_into().unwrap());

            for i in 0..page.header.tuple_count {
                if let Some(tuple_data) = page.get_tuple(i) {
                    let (tuple, _) = bincode::decode_from_slice(tuple_data, bincode::config::standard()).map_err(|e| e.to_string())?;
                    result_tuples.push(tuple);
                }
            }
        }

        prefetch_handle.await;

        Ok(ExecutionResult::Data(result_tuples))
    }
}
