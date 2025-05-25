use std::any::Any;
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use crate::heap::{Heap, HeapError, HeapResult};

/// A standard heap implementation using a mark-sweep GC algorithm
pub struct StandardHeap {
    id: u32,
    objects: RwLock<HashMap<u64, ObjectBox>>,
    next_id: Mutex<u64>,
    gc_enabled: bool,
    gc_threshold: usize,
}

/// A type-erased wrapper for storing objects of different types
struct ObjectBox {
    object: Arc<dyn Any + Send + Sync>,
    size: usize,
    type_id: u32,
    // For GC marking
    marked: bool,
}

impl StandardHeap {
    pub fn new(id: u32) -> HeapResult<Self> {
        Ok(Self {
            id,
            objects: RwLock::new(HashMap::new()),
            next_id: Mutex::new(1),
            gc_enabled: true,
            gc_threshold: 10000, // Default threshold: 10K objects
        })
    }
    
    /// Get the next object ID
    fn next_object_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id = current + 1;
        current
    }
    
    /// Check if garbage collection should run
    fn should_collect(&self) -> bool {
        if !self.gc_enabled {
            return false;
        }
        
        let objects = self.objects.read().unwrap();
        objects.len() >= self.gc_threshold
    }
    
    /// Set the garbage collection threshold
    pub fn set_gc_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
    }
    
    /// Enable or disable garbage collection
    pub fn set_gc_enabled(&mut self, enabled: bool) {
        self.gc_enabled = enabled;
    }
}

impl Heap for StandardHeap {
    fn allocate_object(&self, object: Box<dyn Any + Send + Sync>, size: usize, type_id: u32) -> HeapResult<u64> {
        // Check if we should run GC
        if self.should_collect() {
            self.collect()?;
        }
        
        let object_id = self.next_object_id();
        
        let object_box = ObjectBox {
            object: Arc::from(object),
            size,
            type_id,
            marked: false,
        };
        
        let mut objects = self.objects.write().unwrap();
        objects.insert(object_id, object_box);
        
        Ok(object_id)
    }
    
    fn get_object_any(&self, object_id: u64) -> HeapResult<Arc<dyn Any + Send + Sync>> {
        let objects = self.objects.read().unwrap();
        
        let object_box = objects.get(&object_id).ok_or_else(|| {
            HeapError::InvalidObjectId(format!("Object ID {} not found", object_id))
        })?;
        
        Ok(object_box.object.clone())
    }
    
    fn collect(&self) -> HeapResult<()> {
        // For simplicity, no actual GC implementation yet
        // In a real implementation, we would:
        // 1. Mark reachable objects (starting from roots)
        // 2. Sweep unmarked objects
        Ok(())
    }
    
    fn object_count(&self) -> usize {
        let objects = self.objects.read().unwrap();
        objects.len()
    }
    
    fn memory_usage(&self) -> usize {
        let objects = self.objects.read().unwrap();
        objects.values().map(|obj| obj.size).sum()
    }
    
    fn id(&self) -> u32 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::HeapExt;
    use crate::memory::MemoryObject;
    use std::sync::Arc;

    // Test struct implementing MemoryObject
    #[derive(Debug, Clone, PartialEq)]
    struct TestObject {
        data: String,
        size: usize,
        type_id: u32,
    }

    impl TestObject {
        fn new(data: String, type_id: u32) -> Self {
            let size = data.len() + std::mem::size_of::<String>() + 8; // Rough size calculation
            Self { data, size, type_id }
        }
    }

    impl MemoryObject for TestObject {
        fn size(&self) -> usize {
            self.size
        }

        fn type_id(&self) -> u32 {
            self.type_id
        }
    }

    // Another test struct for type testing
    #[derive(Debug, Clone, PartialEq)]
    struct NumberObject {
        value: i64,
    }

    impl MemoryObject for NumberObject {
        fn size(&self) -> usize {
            std::mem::size_of::<i64>()
        }

        fn type_id(&self) -> u32 {
            2
        }
    }

    #[test]
    fn test_standard_heap_creation() {
        let heap_id = 42;
        let heap = StandardHeap::new(heap_id).unwrap();
        
        assert_eq!(heap.id(), heap_id);
        assert_eq!(heap.object_count(), 0);
        assert_eq!(heap.memory_usage(), 0);
    }

    #[test]
    fn test_standard_heap_allocate_object() {
        let heap = StandardHeap::new(1).unwrap();
        let test_obj = TestObject::new("Hello, World!".to_string(), 1);
        let expected_size = test_obj.size();
        
        let object_id = heap.allocate(test_obj.clone()).unwrap();
        
        assert_eq!(object_id, 1); // First object should get ID 1
        assert_eq!(heap.object_count(), 1);
        assert_eq!(heap.memory_usage(), expected_size);
    }

    #[test]
    fn test_standard_heap_get_object() {
        let heap = StandardHeap::new(1).unwrap();
        let test_obj = TestObject::new("Test Data".to_string(), 1);
        
        let object_id = heap.allocate(test_obj.clone()).unwrap();
        let retrieved_obj = heap.get_object::<TestObject>(object_id).unwrap();
        
        assert_eq!(*retrieved_obj, test_obj);
    }

    #[test]
    fn test_standard_heap_multiple_objects() {
        let heap = StandardHeap::new(1).unwrap();
        
        let obj1 = TestObject::new("Object 1".to_string(), 1);
        let obj2 = TestObject::new("Object 2".to_string(), 1);
        let obj3 = NumberObject { value: 42 };
        
        let id1 = heap.allocate(obj1.clone()).unwrap();
        let id2 = heap.allocate(obj2.clone()).unwrap();
        let id3 = heap.allocate(obj3.clone()).unwrap();
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(heap.object_count(), 3);
        
        // Verify we can retrieve all objects
        let retrieved1 = heap.get_object::<TestObject>(id1).unwrap();
        let retrieved2 = heap.get_object::<TestObject>(id2).unwrap();
        let retrieved3 = heap.get_object::<NumberObject>(id3).unwrap();
        
        assert_eq!(*retrieved1, obj1);
        assert_eq!(*retrieved2, obj2);
        assert_eq!(*retrieved3, obj3);
    }

    #[test]
    fn test_standard_heap_invalid_object_id() {
        let heap = StandardHeap::new(1).unwrap();
        
        let result = heap.get_object::<TestObject>(999);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            HeapError::InvalidObjectId(msg) => {
                assert!(msg.contains("Object ID 999 not found"));
            },
            _ => panic!("Expected InvalidObjectId error"),
        }
    }

    #[test]
    fn test_standard_heap_type_mismatch() {
        let heap = StandardHeap::new(1).unwrap();
        let test_obj = TestObject::new("Test".to_string(), 1);
        
        let object_id = heap.allocate(test_obj).unwrap();
        
        // Try to retrieve as wrong type
        let result = heap.get_object::<NumberObject>(object_id);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            HeapError::TypeMismatch => {},
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_standard_heap_memory_usage_calculation() {
        let heap = StandardHeap::new(1).unwrap();
        
        let obj1 = TestObject::new("Short".to_string(), 1);
        let obj2 = TestObject::new("A much longer string for testing".to_string(), 1);
        let obj3 = NumberObject { value: 123 };
        
        let expected_total = obj1.size() + obj2.size() + obj3.size();
        
        heap.allocate(obj1).unwrap();
        heap.allocate(obj2).unwrap();
        heap.allocate(obj3).unwrap();
        
        assert_eq!(heap.memory_usage(), expected_total);
    }

    #[test]
    fn test_standard_heap_collect_garbage() {
        let heap = StandardHeap::new(1).unwrap();
        
        // Add some objects
        let obj1 = TestObject::new("Object 1".to_string(), 1);
        let obj2 = TestObject::new("Object 2".to_string(), 1);
        
        heap.allocate(obj1).unwrap();
        heap.allocate(obj2).unwrap();
        
        assert_eq!(heap.object_count(), 2);
        
        // Trigger garbage collection (currently a no-op)
        let result = heap.collect();
        assert!(result.is_ok());
        
        // Objects should still be there since GC is not implemented yet
        assert_eq!(heap.object_count(), 2);
    }

    #[test]
    fn test_standard_heap_gc_configuration() {
        let mut heap = StandardHeap::new(1).unwrap();
        
        // Test setting GC threshold
        heap.set_gc_threshold(100);
        heap.set_gc_enabled(false);
        
        // With GC disabled, should_collect should return false
        assert!(!heap.should_collect());
        
        // Enable GC
        heap.set_gc_enabled(true);
        
        // Add objects up to threshold
        for i in 0..101 {
            let obj = TestObject::new(format!("Object {}", i), 1);
            heap.allocate(obj).unwrap();
        }
        
        // Should trigger GC at threshold
        assert!(heap.should_collect());
    }

    #[test]
    fn test_standard_heap_concurrent_allocation() {
        use std::thread;
        
        let heap = Arc::new(StandardHeap::new(1).unwrap());
        let mut handles = vec![];
        
        // Spawn multiple threads to allocate objects concurrently
        for i in 0..10 {
            let heap_clone = Arc::clone(&heap);
            let handle = thread::spawn(move || {
                let obj = TestObject::new(format!("Thread {} object", i), 1);
                heap_clone.allocate(obj)
            });
            handles.push(handle);
        }
        
        let mut object_ids = vec![];
        for handle in handles {
            let object_id = handle.join().unwrap().unwrap();
            object_ids.push(object_id);
        }
        
        // All allocations should succeed and get unique IDs
        assert_eq!(object_ids.len(), 10);
        assert_eq!(heap.object_count(), 10);
        
        // All IDs should be unique
        object_ids.sort();
        for i in 0..object_ids.len() - 1 {
            assert_ne!(object_ids[i], object_ids[i + 1], "Found duplicate object ID: {}", object_ids[i]);
        }
    }

    #[test]
    fn test_standard_heap_get_object_any() {
        let heap = StandardHeap::new(1).unwrap();
        let test_obj = TestObject::new("Any test".to_string(), 1);
        
        let object_id = heap.allocate(test_obj.clone()).unwrap();
        let any_ref = heap.get_object_any(object_id).unwrap();
        
        // Try to downcast back to the original type
        let downcast_result = any_ref.downcast::<TestObject>();
        assert!(downcast_result.is_ok());
        
        let retrieved_obj = downcast_result.unwrap();
        assert_eq!(*retrieved_obj, test_obj);
    }

    #[test]
    fn test_standard_heap_next_object_id_sequence() {
        let heap = StandardHeap::new(1).unwrap();
        
        // Allocate several objects and verify ID sequence
        let obj = TestObject::new("Test".to_string(), 1);
        
        let id1 = heap.allocate(obj.clone()).unwrap();
        let id2 = heap.allocate(obj.clone()).unwrap();
        let id3 = heap.allocate(obj.clone()).unwrap();
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_standard_heap_empty_string_object() {
        let heap = StandardHeap::new(1).unwrap();
        let empty_obj = TestObject::new("".to_string(), 1);
        
        let object_id = heap.allocate(empty_obj.clone()).unwrap();
        let retrieved = heap.get_object::<TestObject>(object_id).unwrap();
        
        assert_eq!(*retrieved, empty_obj);
        assert_eq!(retrieved.data, "");
    }

    #[test]
    fn test_standard_heap_large_object() {
        let heap = StandardHeap::new(1).unwrap();
        let large_string = "x".repeat(10000);
        let large_obj = TestObject::new(large_string.clone(), 1);
        let expected_size = large_obj.size();
        
        let object_id = heap.allocate(large_obj.clone()).unwrap();
        let retrieved = heap.get_object::<TestObject>(object_id).unwrap();
        
        assert_eq!(*retrieved, large_obj);
        assert_eq!(heap.memory_usage(), expected_size);
        assert!(heap.memory_usage() > 10000); // Should be at least the string size
    }
}