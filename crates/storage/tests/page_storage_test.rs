use sqlrustgo_storage::{PageStorageBufferPool, PageStorageEngine, FileStorageEngine, MemoryStorageEngine, StorageError, PageId};
use sqlrustgo_storage::PageStoragePage;
use std::path::Path;

#[test]
fn test_page_creation() {
    let page = PageStoragePage::new(1);
    assert_eq!(page.get_page_id(), 1);
    assert!(!page.is_dirty());
}

#[test]
fn test_page_data_access() {
    let mut page = PageStoragePage::new(1);
    page.data[0] = 0x42;
    page.data[1] = 0x43;
    assert_eq!(page.get_data()[0], 0x42);
    assert_eq!(page.get_data()[1], 0x43);
}

#[test]
fn test_page_set_dirty() {
    let mut page = PageStoragePage::new(1);
    assert!(!page.is_dirty());
    page.set_dirty(true);
    assert!(page.is_dirty());
    page.set_dirty(false);
    assert!(!page.is_dirty());
}

#[test]
fn test_buffer_pool_basic() {
    let mut pool = PageStorageBufferPool::new(10);
    assert_eq!(pool.capacity(), 10);
    assert_eq!(pool.size(), 0);
}

#[test]
fn test_buffer_pool_get_page() {
    let mut pool = PageStorageBufferPool::new(10);
    let page = pool.get(1).unwrap();
    assert_eq!(page.lock().unwrap().get_page_id(), 1);
    assert_eq!(pool.size(), 1);
}

#[test]
fn test_buffer_pool_evict() {
    let mut pool = PageStorageBufferPool::new(2);
    pool.get(1).unwrap();
    pool.get(2).unwrap();
    assert_eq!(pool.size(), 2);
    
    let evicted = pool.evict();
    assert!(evicted.is_some());
    assert_eq!(pool.size(), 1);
}

#[test]
fn test_memory_storage_engine() {
    let mut storage = MemoryStorageEngine::new();
    
    // Create a page
    let mut page = PageStoragePage::new(1);
    page.data[0] = 0x42;
    
    // Write page
    storage.write_page(&page).unwrap();
    
    // Read page
    let read_page = storage.read_page(1).unwrap();
    assert_eq!(read_page.get_page_id(), 1);
    assert_eq!(read_page.get_data()[0], 0x42);
    
    // Flush
    storage.flush().unwrap();
}

#[test]
fn test_file_storage_engine() {
    // Create a temporary file
    let temp_file = std::env::temp_dir().join("test_page_storage.db");
    
    // Create storage engine
    let mut storage = FileStorageEngine::new(temp_file.as_path()).unwrap();
    
    // Create a page
    let mut page = PageStoragePage::new(1);
    page.data[0] = 0x42;
    
    // Write page
    storage.write_page(&page).unwrap();
    
    // Read page
    let read_page = storage.read_page(1).unwrap();
    assert_eq!(read_page.get_page_id(), 1);
    assert_eq!(read_page.get_data()[0], 0x42);
    
    // Flush
    storage.flush().unwrap();
    
    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_storage_engine_error_handling() {
    let mut storage = MemoryStorageEngine::new();
    
    // Try to read a non-existent page
    let result = storage.read_page(999);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StorageError::PageNotFound);
}

#[test]
fn test_memory_storage_engine_multiple_pages() {
    let mut storage = MemoryStorageEngine::new();
    
    // Write multiple pages
    for i in 0..5 {
        let mut page = PageStoragePage::new(i);
        page.data[0] = (i + 1) as u8;
        storage.write_page(&page).unwrap();
    }
    
    // Read pages back
    for i in 0..5 {
        let read_page = storage.read_page(i).unwrap();
        assert_eq!(read_page.get_page_id(), i);
        assert_eq!(read_page.get_data()[0], (i + 1) as u8);
    }
}

#[test]
fn test_file_storage_engine_persistence() {
    // Create a temporary file
    let temp_file = std::env::temp_dir().join("test_page_persistence.db");
    
    // First session: write data
    {
        let mut storage = FileStorageEngine::new(temp_file.as_path()).unwrap();
        let mut page = PageStoragePage::new(1);
        page.data[0] = 0x42;
        storage.write_page(&page).unwrap();
        storage.flush().unwrap();
    }
    
    // Second session: read data back
    {
        let mut storage = FileStorageEngine::new(temp_file.as_path()).unwrap();
        let read_page = storage.read_page(1).unwrap();
        assert_eq!(read_page.get_page_id(), 1);
        assert_eq!(read_page.get_data()[0], 0x42);
    }
    
    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}
