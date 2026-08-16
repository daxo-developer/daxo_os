use alloc::vec::Vec;
use alloc::format;

pub const SECTOR_SIZE: usize = 512;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DirectoryEntry {
    pub name: [u8; 24],
    pub start_block: u32,
    pub size: u32,
    pub flags: u8,
}

impl DirectoryEntry {
    pub fn new(name_str: &str, start_block: u32, size: u32) -> Self {
        let mut name = [0u8; 24];
        let bytes = name_str.as_bytes();
        let len = core::cmp::min(bytes.len(), 23);
        name[..len].copy_from_slice(&bytes[..len]);

        Self {
            name,
            start_block,
            size,
            flags: 0x01,
        }
    }

    pub fn name_as_str(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("INVALID")
    }
}

pub struct SimpleFS;

impl SimpleFS {
    pub fn init() {
        crate::println!("[FS] Initializing Simple FAT-like File System.");
        crate::println!("[FS] File System initialized successfully.");
    }

    pub fn list_directory() {
        crate::println!("=== DaxoOS File System Listing ===");
        crate::println!("File: kernel.txt | Sector: 10 | Size: 1024B");
    }

    pub fn read_file(name: &str) -> Option<Vec<u8>> {
        if name == "kernel.txt" {
            let mock_data = format!("Hello from file {} stored on ATA drive!", name);
            return Some(mock_data.into_bytes());
        }
        None
    }
}
