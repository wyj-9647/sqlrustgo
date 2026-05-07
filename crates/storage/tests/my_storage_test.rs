use sqlrustgo_storage::page::{Page, PageId, PAGE_SIZE};
use sqlrustgo_storage::buffer_pool::{BufferPool, BufferPoolError};
use sqlrustgo_storage::engine::{StorageEngine, FileStorageEngine, MemoryStorageEngine, StorageError};
use std::path::Path;
use std::fs;

#[test]
fn test_page_creation() {
    let page_id: PageId = 1;
    let page = Page::new(page_id);
    assert_eq!(page.page_id, page_id);
    assert!(!page.is_dirty);
    assert_eq!(page.data.len(), PAGE_SIZE);
}

#[test]
fn test_buffer_pool() {
    let mut buffer_pool = BufferPool::new(5);

    // Test getting a new page
    let page_id: PageId = 1;
    let page = buffer_pool.get(page_id).unwrap();
    assert_eq!(page.lock().unwrap().page_id, page_id);
    assert_eq!(buffer_pool.size(), 1);
    assert_eq!(buffer_pool.capacity(), 5);

    // Test getting an existing page
    let page2 = buffer_pool.get(page_id).unwrap();
    assert_eq!(page2.lock().unwrap().page_id, page_id);
    assert_eq!(buffer_pool.size(), 1);

    // Test buffer full
    for i in 2..7 {
        buffer_pool.get(i as PageId).unwrap();
    }
    assert_eq!(buffer_pool.size(), 5);

    // This should fail because buffer is full
    let result = buffer_pool.get(7 as PageId);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BufferPoolError::BufferFull);

    // Test eviction
    let evicted = buffer_pool.evict();
    assert!(evicted.is_some());
    assert_eq!(buffer_pool.size(), 4);

    // Test contains
    assert!(buffer_pool.contains(2));
    assert!(!buffer_pool.contains(1));
}

#[test]
fn test_memory_storage_engine() {
    let mut engine = MemoryStorageEngine::new();

    // Test writing a page
    let page_id: PageId = 1;
    let mut page = Page::new(page_id);
    page.data[0] = 42;
    page.set_dirty(true);

    engine.write_page(&page).unwrap();

    // Test reading the page back
    let read_page = engine.read_page(page_id).unwrap();
    assert_eq!(read_page.page_id, page_id);
    assert_eq!(read_page.data[0], 42);

    // Test reading a non-existent page
    let result = engine.read_page(999);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StorageError::PageNotFound);

    // Test flushing
    engine.flush().unwrap();
}

#[test]
fn test_file_storage_engine() {
    // Create a temporary file
    let temp_path = Path::new("./test_storage.db");

    // Clean up any existing file
    if temp_path.exists() {
        fs::remove_file(temp_path).unwrap();
    }

    // Create a file storage engine
    let mut engine = FileStorageEngine::new(temp_path).unwrap();

    // Test writing a page
    let page_id: PageId = 1;
    let mut page = Page::new(page_id);
    page.data[0] = 42;
    page.set_dirty(true);

    engine.write_page(&page).unwrap();
    engine.flush().unwrap();

    // Test reading the page back
    let read_page = engine.read_page(page_id).unwrap();
    assert_eq!(read_page.page_id, page_id);
    assert_eq!(read_page.data[0], 42);

    // Clean up
    fs::remove_file(temp_path).unwrap();
}