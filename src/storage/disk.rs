use std::io;

use crate::storage::page::{PAGE_SIZE, PageId};
use monoio::fs::{File, OpenOptions};
// use std::os::unix::fs::OpenOptionsExt;
pub struct DiskManager {
    file: File,
}

impl DiskManager {
    pub async fn new(file_path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            // .custom_flags(libc::O_DIRECT) // TODO: 目前直接使用 O_DIRECT 会导致读写失败，后续需要处理对齐问题
            .open(file_path)
            .await?;

        Ok(Self { file })
    }

    pub async fn read_page(
        &self,
        page_id: PageId,
        mut buffer: Vec<u8>,
    ) -> (io::Result<usize>, Vec<u8>) {
        let offset = page_id as u64 * PAGE_SIZE as u64;
        buffer.resize(PAGE_SIZE, 0);

        let (res, buf) = self.file.read_at(buffer, offset).await;
        (res, buf)
    }

    pub async fn write_page(
        &self,
        page_id: PageId,
        buffer: Vec<u8>,
    ) -> (io::Result<usize>, Vec<u8>) {
        let offset = page_id as u64 * PAGE_SIZE as u64;
        let (res, buffer) = self.file.write_at(buffer, offset).await;
        (res, buffer)
    }
}
