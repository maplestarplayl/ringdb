use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::storage::{
    disk::DiskManager,
    page::{PAGE_SIZE, PageId},
    replacer::ClockReplacer,
};

type FrameId = usize;

pub struct Frame {
    pub page_id: PageId,
    pub data: Vec<u8>,
    // pin_count: usize,
    pub is_dirty: bool,
}

pub struct BufferPoolManager {
    _pool_size: usize,
    // disk_manager: Arc<DiskManager>,
    frames: Arc<RwLock<Vec<Frame>>>,
    page_table: Arc<Mutex<HashMap<PageId, FrameId>>>,
    replacer: Arc<ClockReplacer>,
}

pub struct PageGuard {
    bpm: Arc<BufferPoolManager>,
    frame_id: FrameId,
}

pub struct PageWriteGuard {
    bpm: Arc<BufferPoolManager>,
    frame_id: FrameId,
}

impl BufferPoolManager {
    pub fn new(pool_size: usize) -> Arc<Self> {
        let frames = (0..pool_size)
            .map(|_| Frame {
                page_id: u32::MAX,
                is_dirty: false,
                data: vec![0; PAGE_SIZE],
            })
            .collect();
        Arc::new(Self {
            _pool_size: pool_size,
            // disk_manager,
            frames: Arc::new(RwLock::new(frames)),
            page_table: Arc::new(Mutex::new(HashMap::new())),
            replacer: Arc::new(ClockReplacer::new(pool_size)),
        })
    }

    pub async fn fetch_page(self: &Arc<Self>, page_id: PageId, disk_manager: Arc<DiskManager>) -> Result<PageGuard, String> {
        let frame_id = self.get_frame_for_page(page_id, disk_manager).await?;
        Ok(PageGuard {
            bpm: self.clone(),
            frame_id,
        })
    }

    pub async fn fetch_page_mut(self: &Arc<Self>, page_id: PageId, disk_manager: Arc<DiskManager>) -> Result<PageWriteGuard, String> {
        let frame_id = self.get_frame_for_page(page_id, disk_manager).await?;
        Ok(PageWriteGuard {
            bpm: self.clone(),
            frame_id,
        })
    }

    async fn get_frame_for_page(&self, page_id: PageId, disk_manager: Arc<DiskManager>) -> Result<usize, String> {
        {
            let page_table = self.page_table.lock().unwrap();
            if let Some(&frame_id) = page_table.get(&page_id) {
                self.replacer.pin(frame_id);
                return Ok(frame_id);
            }
        }

        let frame_id = self.replacer.victim().ok_or("No free frame available")?;
        self.replacer.pin(frame_id);

        let (old_page_id, is_dirty, mut data_buf) = {
            let mut frames = self.frames.write().unwrap();
            let victim_frame = &mut frames[frame_id];
            let old_page_id = victim_frame.page_id;
            let is_dirty = victim_frame.is_dirty;
            // Postpone modifying the frame's metadata until after I/O
            let data_buf = std::mem::take(&mut victim_frame.data);

            (old_page_id, is_dirty, data_buf)
        };

        if is_dirty {
            let (res, buf) = disk_manager.write_page(old_page_id, data_buf).await;
            res.map_err(|e| format!("Failed to write page {}: {}", old_page_id, e))?;
            data_buf = buf;
        }
        data_buf.fill(0);

        let (res, buf) = disk_manager.read_page(page_id, data_buf).await;
        res.map_err(|e| format!("Failed to read page {}: {}", page_id, e))?;

        {
            let mut frames = self.frames.write().unwrap();
            let mut page_table = self.page_table.lock().unwrap();

            if old_page_id != u32::MAX {
                page_table.remove(&old_page_id);
            }

            let victim_frame = &mut frames[frame_id];
            victim_frame.page_id = page_id;
            victim_frame.is_dirty = false;
            victim_frame.data = buf;

            page_table.insert(page_id, frame_id);
        }

        Ok(frame_id)
    }
}

impl PageGuard {
    pub fn frame_id(&self) -> usize {
        self.frame_id
    }
    pub fn page_id(&self) -> u32 {
        let frames = self.bpm.frames.read().unwrap();
        frames[self.frame_id].page_id
    }

    pub fn get<'a>(&'a self) -> RwLockReadGuard<'a, Vec<Frame>> {
        self.bpm.frames.read().unwrap()
    }
}
impl Drop for PageGuard {
    fn drop(&mut self) {
        self.bpm.replacer.unpin(self.frame_id);
    }
}
impl PageWriteGuard {
    pub fn frame_id(&self) -> usize {
        self.frame_id
    }
    pub fn page_id(&self) -> u32 {
        let frames = self.bpm.frames.read().unwrap();
        frames[self.frame_id].page_id
    }

    pub fn get_mut<'a>(&'a mut self) -> RwLockWriteGuard<'a, Vec<Frame>> {
        self.bpm.frames.write().unwrap()
    }
}
impl Drop for PageWriteGuard {
    fn drop(&mut self) {
        let mut frames = self.bpm.frames.write().unwrap();
        frames[self.frame_id].is_dirty = true;
        self.bpm.replacer.unpin(self.frame_id);
    }
}
