use sqlrustgo_storage::{
    page::{Page, PageType, PAGE_SIZE, PAGE_HEADER_SIZE, PAGE_DATA_SIZE},
    buffer_pool::BufferPool,
    engine::{StorageEngine, MemoryStorage, MemoryStorageEngine, FileStorageEngine, StorageError, ColumnDefinition, TableInfo},
    PageStoragePage, PageStorageBufferPool, PageStorageEngine, FileStorageEngine as PageFileStorageEngine, MemoryStorageEngine as PageMemoryStorageEngine, StorageError as PageStorageError, PageId,
};
use std::sync::Arc;

#[test]
fn test_page_constants() {
    assert_eq!(PAGE_SIZE, 4096);
    assert_eq!(PAGE_HEADER_SIZE, 64);
    assert_eq!(PAGE_DATA_SIZE, 4032);
}

#[test]
fn test_page_type_variants() {
    assert_eq!(PageType::Data as u8, 1);
    assert_eq!(PageType::Index as u8, 2);
    assert_eq!(PageType::Free as u8, 0);
    assert_eq!(PageType::Meta as u8, 3);
}

#[test]
fn test_page_new() {
    let page = Page::new(1);
    assert_eq!(page.page_id(), 1);
    assert_eq!(page.page_type(), PageType::Free);
    assert_eq!(page.row_count(), 0);
    assert_eq!(page.free_space(), PAGE_DATA_SIZE as u32);
    assert!(page.verify_checksum());
}

#[test]
fn test_page_new_data() {
    let page = Page::new_data(1, 100);
    assert_eq!(page.page_id(), 1);
    assert_eq!(page.page_type(), PageType::Data);
    assert!(page.verify_checksum());
}

#[test]
fn test_page_checksum() {
    let mut page = Page::new_data(1, 100);
    let initial_checksum = page.checksum();
    assert!(initial_checksum != 0);
    assert!(page.verify_checksum());
}

#[test]
fn test_page_checksum_corruption() {
    let mut page = Page::new_data(1, 100);
    page.data[100] = 0xFF;
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
    assert_eq!(rows[0], row);
}

#[test]
fn test_page_write_multiple_rows() {
    use sqlrustgo_types::Value;

    let mut page = Page::new_data(1, 100);
    for i in 0..10 {
        let row = vec![Value::Integer(i), Value::Text(format!("row_{}", i))];
        assert!(page.write_row(&row));
    }
    assert_eq!(page.row_count(), 10);

    let rows = page.read_rows();
    assert_eq!(rows.len(), 10);
}

#[test]
fn test_page_full_space() {
    use sqlrustgo_types::Value;

    let mut page = Page::new_data(1, 100);
    let large_row = vec![Value::Text("x".repeat(4000))];

    let mut count = 0;
    while page.write_row(&large_row) {
        count += 1;
    }
    assert!(count > 0);
}

#[test]
fn test_page_to_from_bytes() {
    use sqlrustgo_types::Value;

    let mut page = Page::new_data(1, 100);
    page.write_row(&vec![Value::Integer(42)]);

    let bytes = page.to_bytes();
    assert_eq!(bytes.len(), PAGE_SIZE);

    let restored = Page::from_bytes(bytes).unwrap();
    assert_eq!(restored.page_id(), 1);
    assert_eq!(restored.row_count(), 1);
}

#[test]
fn test_page_from_bytes_invalid() {
    let result = Page::from_bytes(vec![0u8; 100]);
    assert!(result.is_none());
}

#[test]
fn test_buffer_pool_full_cycle() {
    let mut pool = BufferPool::new(3);

    let page1 = pool.allocate(1);
    page1.lock().unwrap().data[0] = 0x11;

    let page2 = pool.allocate(2);
    page2.lock().unwrap().data[0] = 0x22;

    let page3 = pool.allocate(3);
    page3.lock().unwrap().data[0] = 0x33;

    assert_eq!(pool.len(), 3);
    assert!(pool.get(1).is_some());
    assert!(pool.get(2).is_some());
    assert!(pool.get(3).is_some());

    let page4 = pool.allocate(4);
    assert!(pool.get(4).is_some());
}

#[test]
fn test_buffer_pool_lru_order() {
    let pool = BufferPool::new(3);

    pool.insert(Arc::new(Page::new(1)));
    pool.insert(Arc::new(Page::new(2)));
    pool.insert(Arc::new(Page::new(3)));

    let _ = pool.get(1);

    pool.insert(Arc::new(Page::new(4)));

    assert!(pool.get(1).is_some());
    assert!(pool.get(2).is_none());
}

#[test]
fn test_buffer_pool_stats() {
    let pool = BufferPool::new(10);
    pool.insert(Arc::new(Page::new(1)));

    let _ = pool.get(1);
    let _ = pool.get(999);

    let stats = pool.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_buffer_pool_hit_rate() {
    let pool = BufferPool::new(10);
    pool.insert(Arc::new(Page::new(1)));

    for _ in 0..9 {
        let _ = pool.get(1);
    }
    let _ = pool.get(999);

    let rate = pool.hit_rate();
    assert!((rate - 0.9).abs() < 0.01);
}

#[test]
fn test_buffer_pool_prefetch() {
    let pool = BufferPool::with_prefetch(10, 5);

    pool.prefetch_range(1, 3, |id| Arc::new(Page::new(id)));

    assert!(pool.get(1).is_some());
    assert!(pool.get(2).is_some());
}

#[test]
fn test_memory_storage_basic() {
    let mut storage = MemoryStorage::new();
    assert!(storage.list_tables().is_empty());
    assert!(!storage.has_table("users"));
}

#[test]
fn test_memory_storage_create_table() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
        }],
    };

    storage.create_table(&info).unwrap();
    assert!(storage.has_table("users"));
}

#[test]
fn test_memory_storage_insert_and_scan() {
    use sqlrustgo_types::Value;

    let mut storage = MemoryStorage::new();
    storage.create_table(&TableInfo {
        name: "users".to_string(),
        columns: vec![],
    }).unwrap();

    let records = vec![
        vec![Value::Integer(1), Value::Text("Alice".to_string())],
        vec![Value::Integer(2), Value::Text("Bob".to_string())],
    ];

    storage.insert("users", records).unwrap();
    let result = storage.scan("users").unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_memory_storage_delete() {
    use sqlrustgo_types::Value;

    let mut storage = MemoryStorage::new();
    storage.create_table(&TableInfo {
        name: "users".to_string(),
        columns: vec![],
    }).unwrap();

    storage.insert("users", vec![vec![Value::Integer(1)]]).unwrap();
    let count = storage.delete("users", &[]).unwrap();
    assert_eq!(count, 1);

    let result = storage.scan("users").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_memory_storage_update() {
    use sqlrustgo_types::Value;

    let mut storage = MemoryStorage::new();
    storage.create_table(&TableInfo {
        name: "users".to_string(),
        columns: vec![],
    }).unwrap();

    storage.insert("users", vec![vec![Value::Integer(1)]]).unwrap();
    let count = storage.update("users", &[], &[(0, Value::Integer(2))]).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_memory_storage_drop_table() {
    let mut storage = MemoryStorage::new();
    storage.create_table(&TableInfo {
        name: "users".to_string(),
        columns: vec![],
    }).unwrap();

    storage.drop_table("users").unwrap();
    assert!(!storage.has_table("users"));
}

#[test]
fn test_page_storage_page_new() {
    let page = PageStoragePage::new(1);
    assert_eq!(page.get_page_id(), 1);
    assert!(!page.is_dirty());
}

#[test]
fn test_page_storage_page_dirty() {
    let mut page = PageStoragePage::new(1);
    assert!(!page.is_dirty());
    page.set_dirty(true);
    assert!(page.is_dirty());
}

#[test]
fn test_page_storage_buffer_pool() {
    let mut pool = PageStorageBufferPool::new(5);
    assert_eq!(pool.capacity(), 5);
    assert_eq!(pool.size(), 0);
}

#[test]
fn test_page_storage_buffer_pool_put_get() {
    let mut pool = PageStorageBufferPool::new(5);
    let page = pool.get(1).unwrap();
    assert_eq!(pool.size(), 1);
    assert!(pool.contains(1));
}

#[test]
fn test_page_storage_buffer_pool_evict() {
    let mut pool = PageStorageBufferPool::new(2);
    pool.get(1).unwrap();
    pool.get(2).unwrap();

    let evicted = pool.evict();
    assert!(evicted.is_some());
    assert_eq!(pool.size(), 1);
}

#[test]
fn test_page_memory_storage_engine() {
    let mut storage = PageMemoryStorageEngine::new();

    let mut page = PageStoragePage::new(1);
    page.data[0] = 0x42;

    storage.write_page(&page).unwrap();

    let read = storage.read_page(1).unwrap();
    assert_eq!(read.get_data()[0], 0x42);

    storage.flush().unwrap();
}

#[test]
fn test_page_file_storage_engine() {
    let temp_file = std::env::temp_dir().join("test_comprehensive.db");

    let mut storage = PageFileStorageEngine::new(temp_file.as_path()).unwrap();

    let mut page = PageStoragePage::new(1);
    page.data[0] = 0x42;

    storage.write_page(&page).unwrap();
    storage.flush().unwrap();

    drop(storage);

    let mut storage2 = PageFileStorageEngine::new(temp_file.as_path()).unwrap();
    let read = storage2.read_page(1).unwrap();
    assert_eq!(read.get_data()[0], 0x42);

    std::fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_page_storage_error_not_found() {
    let mut storage = PageMemoryStorageEngine::new();
    let result = storage.read_page(999);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PageStorageError::PageNotFound);
}

#[test]
fn test_page_storage_multiple_pages() {
    let mut storage = PageMemoryStorageEngine::new();

    for i in 0..100 {
        let mut page = PageStoragePage::new(i);
        page.data[0] = (i % 256) as u8;
        storage.write_page(&page).unwrap();
    }

    for i in 0..100 {
        let read = storage.read_page(i).unwrap();
        assert_eq!(read.get_data()[0], (i % 256) as u8);
    }
}
