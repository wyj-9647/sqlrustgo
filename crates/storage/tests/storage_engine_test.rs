use sqlrustgo_storage::{page::Page, buffer_pool::BufferPool, engine::{StorageEngine, MemoryStorageEngine, FileStorageEngine, StorageError}};
use std::path::Path;

#[test]
fn test_page_creation() {
    let page = Page::new(1);
    assert_eq!(page.page_id(), 1);
    assert_eq!(page.page_type(), sqlrustgo_storage::page::PageType::Free);
}

#[test]
fn test_page_data_access() {
    let mut page = Page::new_data(1, 100);
    page.data[64] = 0xAB;
    page.data[65] = 0xCD;
    assert_eq!(page.data[64], 0xAB);
    assert_eq!(page.data[65], 0xCD);
}

#[test]
fn test_buffer_pool_basic() {
    let pool = BufferPool::new(10);
    assert_eq!(pool.capacity(), 10);
    assert!(pool.is_empty());
}

#[test]
fn test_buffer_pool_get_page() {
    let pool = BufferPool::new(10);
    let page = std::sync::Arc::new(Page::new(1));
    pool.insert(page);
    let retrieved = pool.get(1);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().page_id(), 1);
}

#[test]
fn test_memory_storage_engine() {
    let mut storage = MemoryStorageEngine::new();
    
    // Create a page
    let mut page = Page::new_data(1, 100);
    page.data[64] = 0x42;
    
    // Write page
    storage.write_page(&page).unwrap();
    
    // Read page
    let read_page = storage.read_page(1).unwrap();
    assert_eq!(read_page.page_id(), 1);
    assert_eq!(read_page.data[64], 0x42);
    
    // Flush
    storage.flush().unwrap();
}

#[test]
fn test_file_storage_engine() {
    // Create a temporary file
    let temp_file = std::env::temp_dir().join("test_storage.db");
    
    // Create storage engine
    let mut storage = FileStorageEngine::new(temp_file.as_path()).unwrap();
    
    // Create a page
    let mut page = Page::new_data(1, 100);
    page.data[64] = 0x42;
    
    // Write page
    storage.write_page(&page).unwrap();
    
    // Read page
    let read_page = storage.read_page(1).unwrap();
    assert_eq!(read_page.page_id(), 1);
    assert_eq!(read_page.data[64], 0x42);
    
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
fn test_buffer_pool_lru_eviction() {
    let pool = BufferPool::new(3);
    
    // Insert 3 pages
    pool.insert(std::sync::Arc::new(Page::new(1)));
    pool.insert(std::sync::Arc::new(Page::new(2)));
    pool.insert(std::sync::Arc::new(Page::new(3)));
    
    // Access page 1 to make it most recently used
    let _ = pool.get(1);
    
    // Insert new page - should evict page 2 (LRU)
    pool.insert(std::sync::Arc::new(Page::new(4)));
    
    assert!(pool.get(1).is_some()); // Should exist (was accessed)
    assert!(pool.get(2).is_none()); // Should be evicted
    assert!(pool.get(3).is_some());
    assert!(pool.get(4).is_some());
}

#[test]
fn test_page_checksum() {
    let mut page = Page::new_data(1, 100);
    
    // Write a row to modify page data
    use sqlrustgo_types::Value;
    let row = vec![Value::Integer(42), Value::Text("test".to_string())];
    page.write_row(&row);
    
    // Verify checksum is calculated
    let checksum = page.checksum();
    assert!(checksum != 0);
    
    // Verify checksum is valid
    assert!(page.verify_checksum());
    
    // Modify data to make checksum invalid
    page.data[100] = 0xFF;
    
    // Verify checksum should fail
    assert!(!page.verify_checksum());
}

#[test]
fn test_page_write_read_row() {
    use sqlrustgo_types::Value;
    
    let mut page = Page::new_data(1, 100);
    let row = vec![Value::Integer(1), Value::Text("test".to_string())];
    
    assert!(page.write_row(&row));
    assert_eq!(page.row_count(), 1);
    
    let rows = page.read_rows();
    assert_eq!(rows.len(), 1);
}

#[test]
fn test_memory_storage_engine_multiple_pages() {
    let mut storage = MemoryStorageEngine::new();
    
    // Write multiple pages
    for i in 0..5 {
        let mut page = Page::new_data(i, 100);
        page.data[64] = (i + 1) as u8;
        storage.write_page(&page).unwrap();
    }
    
    // Read pages back
    for i in 0..5 {
        let read_page = storage.read_page(i).unwrap();
        assert_eq!(read_page.page_id(), i);
        assert_eq!(read_page.data[64], (i + 1) as u8);
    }
}

#[test]
fn test_file_storage_engine_persistence() {
    // Create a temporary file
    let temp_file = std::env::temp_dir().join("test_persistence.db");
    
    // First session: write data
    {
        let mut storage = FileStorageEngine::new(temp_file.as_path()).unwrap();
        let mut page = Page::new_data(1, 100);
        page.data[64] = 0x42;
        storage.write_page(&page).unwrap();
        storage.flush().unwrap();
    }
    
    // Second session: read data back
    {
        let mut storage = FileStorageEngine::new(temp_file.as_path()).unwrap();
        let read_page = storage.read_page(1).unwrap();
        assert_eq!(read_page.page_id(), 1);
        assert_eq!(read_page.data[64], 0x42);
    }
    
    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}
