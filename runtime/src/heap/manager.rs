use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use crate::heap::{Heap, HeapResult, StandardHeap};

/// The HeapManager is responsible for creating and tracking all heaps
pub struct HeapManager {
    next_id: AtomicU32,
}

impl HeapManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(1),
        }
    }
    
    /// Generate the next heap ID
    pub fn next_heap_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Create a new heap with the specified ID
    pub fn create_heap(&self, heap_id: u32) -> HeapResult<Arc<dyn Heap>> {
        // For now we only support StandardHeap, but this is where we could create
        // different types of heaps based on configuration
        let heap = StandardHeap::new(heap_id)?;
        Ok(Arc::new(heap))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_manager_creation() {
        let manager = HeapManager::new();
        // The manager should start with next_id of 1
        assert_eq!(manager.next_heap_id(), 1);
        assert_eq!(manager.next_heap_id(), 2);
        assert_eq!(manager.next_heap_id(), 3);
    }

    #[test]
    fn test_heap_manager_create_heap() {
        let manager = HeapManager::new();
        let heap_id = 42;
        
        let result = manager.create_heap(heap_id);
        assert!(result.is_ok());
        
        let heap = result.unwrap();
        assert_eq!(heap.id(), heap_id);
        assert_eq!(heap.object_count(), 0);
        assert_eq!(heap.memory_usage(), 0);
    }

    #[test]
    fn test_heap_manager_multiple_heaps() {
        let manager = HeapManager::new();
        
        let heap1 = manager.create_heap(1).unwrap();
        let heap2 = manager.create_heap(2).unwrap();
        let heap3 = manager.create_heap(3).unwrap();
        
        assert_eq!(heap1.id(), 1);
        assert_eq!(heap2.id(), 2);
        assert_eq!(heap3.id(), 3);
        
        // Each heap should be independent
        assert_eq!(heap1.object_count(), 0);
        assert_eq!(heap2.object_count(), 0);
        assert_eq!(heap3.object_count(), 0);
    }

    #[test]
    fn test_heap_manager_concurrent_id_generation() {
        use std::thread;
        use std::sync::Arc;
        
        let manager = Arc::new(HeapManager::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to test concurrent ID generation
        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                manager_clone.next_heap_id()
            });
            handles.push(handle);
        }
        
        let mut ids = vec![];
        for handle in handles {
            ids.push(handle.join().unwrap());
        }
        
        // All IDs should be unique
        ids.sort();
        for i in 0..ids.len() - 1 {
            assert_ne!(ids[i], ids[i + 1], "Found duplicate ID: {}", ids[i]);
        }
        
        // IDs should be in the range 1-10
        assert_eq!(ids.len(), 10);
        assert!(ids.iter().all(|&id| id >= 1 && id <= 10));
    }
}