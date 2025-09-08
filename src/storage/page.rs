use bincode::{Decode, Encode, config, decode_from_slice, encode_into_slice};

pub const PAGE_SIZE: usize = 8192; // 8KB
pub type PageId = u32;

/// 页头，存储页的元数据
#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct PageHeader {
    pub tuple_count: u16,
}

const HEADER_SIZE: usize = 2; // 2 bytes for u16 tuple_count

/// 一个磁盘页的内存表示
pub struct Page {
    pub header: PageHeader,
    pub data: [u8; PAGE_SIZE],
}

impl Page {
    /// 从原始字节数组中加载一个页
    pub fn from_bytes(bytes: [u8; PAGE_SIZE]) -> Self {
        let (header, _) = decode_from_slice(&bytes[0..HEADER_SIZE], config::standard()).unwrap();
        Self {
            header,
            data: bytes,
        }
    }

    /// 将页内容序列化以便写入磁盘
    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        let mut bytes = self.data;
        encode_into_slice(self.header, &mut bytes[0..HEADER_SIZE], config::standard()).unwrap();
        bytes
    }

    /// 尝试在页中插入一个元组，返回元组的槽位ID。
    /// 这是一个非常简单的实现，仅在末尾追加数据。
    pub fn insert_tuple(&mut self, tuple_data: &[u8]) -> Option<u16> {
        let mut offset = HEADER_SIZE;
        for _ in 0..self.header.tuple_count {
            // Read tuple length (u16, 2 bytes) to find the start of the next one
            let tuple_len =
                u16::from_le_bytes(self.data[offset..offset + 2].try_into().unwrap()) as usize;
            offset += 2 + tuple_len;
        }

        // Check if there is enough space (2 bytes for length + data)
        if offset + 2 + tuple_data.len() > PAGE_SIZE {
            return None;
        }

        // Write length
        let len_bytes = (tuple_data.len() as u16).to_le_bytes();
        self.data[offset..offset + 2].copy_from_slice(&len_bytes);
        // Write data
        self.data[offset + 2..offset + 2 + tuple_data.len()].copy_from_slice(tuple_data);

        let slot_id = self.header.tuple_count;
        self.header.tuple_count += 1;
        Some(slot_id)
    }

    /// 根据槽位ID获取元组的数据切片。
    pub fn get_tuple(&self, slot_id: u16) -> Option<&[u8]> {
        if slot_id >= self.header.tuple_count {
            return None;
        }
        let mut offset = HEADER_SIZE;
        for _ in 0..slot_id {
            let tuple_len =
                u16::from_le_bytes(self.data[offset..offset + 2].try_into().unwrap()) as usize;
            offset += 2 + tuple_len;
        }
        let tuple_len =
            u16::from_le_bytes(self.data[offset..offset + 2].try_into().unwrap()) as usize;
        Some(&self.data[offset + 2..offset + 2 + tuple_len])
    }
}
