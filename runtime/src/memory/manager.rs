use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use crate::memory::{MemoryError, MemoryObject, MemoryResult, ObjectReference};
use crate::heap::{Heap, HeapManager, HeapError};

/// The MemoryManager coordinates all memory operations across multiple heaps
pub struct MemoryManager {
    heaps: DashMap<u32, Arc<dyn Heap>>,
    current_heap_id: RwLock<u32>,
    heap_manager: Arc<HeapManager>,
}

impl From<HeapError> for MemoryError {
    fn from(err: HeapError) -> Self {
        MemoryError::HeapError(format!("{}", err))
    }
}

impl MemoryManager {
    pub fn new() -> Self {
        let heap_manager = Arc::new(HeapManager::new());
        
        // Create default heap with ID 1
        let memory_manager = Self {
            heaps: DashMap::new(),
            current_heap_id: RwLock::new(1),
            heap_manager,
        };
        
        // Initialize the default heap
        memory_manager.create_heap().expect("Failed to create default heap");
        
        memory_manager
    }
    
    /// Create a new heap and return its ID
    pub fn create_heap(&self) -> MemoryResult<u32> {
        let heap_id = self.heap_manager.next_heap_id();
        let heap = self.heap_manager.create_heap(heap_id)?;
        self.heaps.insert(heap_id, heap);
        Ok(heap_id)
    }
    
    /// Switch the current heap to the specified heap ID
    pub fn switch_heap(&self, heap_id: u32) -> MemoryResult<()> {
        if !self.heaps.contains_key(&heap_id) {
            return Err(MemoryError::HeapError(format!("Heap {} does not exist", heap_id)));
        }
        
        let mut current_id = self.current_heap_id.write().unwrap();
        *current_id = heap_id;
        
        Ok(())
    }
    
    /// Get the current heap ID being used for allocations
    pub fn current_heap_id(&self) -> u32 {
        *self.current_heap_id.read().unwrap()
    }
    
    /// Allocate an object in the current heap
    pub fn allocate<T: MemoryObject + 'static>(&self, object: T) -> MemoryResult<ObjectReference> {
        let heap_id = self.current_heap_id();
        
        // Get the current heap
        let heap_ref = self.heaps.get(&heap_id).ok_or_else(|| {
            MemoryError::HeapError(format!("Current heap {} does not exist", heap_id))
        })?;
        
        // Use the heap's allocate_object method directly
        let size = object.size();
        let type_id = object.type_id();
        let object_id = heap_ref.allocate_object(Box::new(object), size, type_id)?;
        
        // Create and return an object reference
        Ok(ObjectReference::new(heap_id, object_id))
    }
    
    /// Get a reference to an object from any heap
    pub fn get_object<T: MemoryObject + 'static>(&self, reference: ObjectReference) -> MemoryResult<Arc<T>> {
        if reference.is_null() {
            return Err(MemoryError::InvalidReference("Cannot dereference null".to_string()));
        }
        
        let heap_id = reference.heap_id();
        let object_id = reference.object_id();
        
        // Get the heap
        let heap_ref = self.heaps.get(&heap_id).ok_or_else(|| {
            MemoryError::InvalidReference(format!("Heap {} does not exist", heap_id))
        })?;
        
        // Get the object as Any first
        let any_obj = heap_ref.get_object_any(object_id)?;
        
        // Then downcast to the requested type
        match Arc::downcast::<T>(any_obj) {
            Ok(typed_obj) => Ok(typed_obj),
            Err(_) => Err(MemoryError::InvalidReference(format!("Type mismatch for object {}", object_id))),
        }
    }
    
    /// Trigger garbage collection on a specific heap
    pub fn collect_heap(&self, heap_id: u32) -> MemoryResult<()> {
        let heap = self.heaps.get(&heap_id).ok_or_else(|| {
            MemoryError::HeapError(format!("Heap {} does not exist", heap_id))
        })?;
        
        Ok(heap.collect()?)
    }
    
    /// Trigger garbage collection on all heaps
    pub fn collect_all(&self) -> MemoryResult<()> {
        for entry in self.heaps.iter() {
            entry.value().collect()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryObject;

    // Test struct implementing MemoryObject
    #[derive(Debug, Clone, PartialEq)]
    struct TestMemoryObject {
        data: String,
        size: usize,
        type_id: u32,
    }

    impl TestMemoryObject {
        fn new(data: String, type_id: u32) -> Self {
            let size = data.len() + std::mem::size_of::<String>() + 8;
            Self { data, size, type_id }
        }
    }

    impl MemoryObject for TestMemoryObject {
        fn size(&self) -> usize {
            self.size
        }

        fn type_id(&self) -> u32 {
            self.type_id
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct NumberObject {
        value: i64,
    }

    impl MemoryObject for NumberObject {
        fn size(&self) -> usize {
            std::mem::size_of::<i64>()
        }

        fn type_id(&self) -> u32 {
            999
        }
    }

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::new();
        
        // Should start with default heap (ID 1)
        assert_eq!(manager.current_heap_id(), 1);
        assert!(manager.heaps.contains_key(&1));
    }

    #[test]
    fn test_memory_manager_create_heap() {
        let manager = MemoryManager::new();
        
        let heap_id = manager.create_heap().unwrap();
        assert_eq!(heap_id, 2); // Second heap should get ID 2
        assert!(manager.heaps.contains_key(&heap_id));
        
        let heap_id2 = manager.create_heap().unwrap();
        assert_eq!(heap_id2, 3); // Third heap should get ID 3
    }

    #[test]
    fn test_memory_manager_switch_heap() {
        let manager = MemoryManager::new();
        let new_heap_id = manager.create_heap().unwrap();
        
        // Switch to new heap
        assert!(manager.switch_heap(new_heap_id).is_ok());
        assert_eq!(manager.current_heap_id(), new_heap_id);
        
        // Switch back to default heap
        assert!(manager.switch_heap(1).is_ok());
        assert_eq!(manager.current_heap_id(), 1);
    }

    #[test]
    fn test_memory_manager_switch_nonexistent_heap() {
        let manager = MemoryManager::new();
        
        let result = manager.switch_heap(999);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MemoryError::HeapError(msg) => {
                assert!(msg.contains("Heap 999 does not exist"));
            },
            _ => panic!("Expected HeapError"),
        }
    }

    #[test]
    fn test_memory_manager_allocate_object() {
        let manager = MemoryManager::new();
        let test_obj = TestMemoryObject::new("Hello, Memory!".to_string(), 1);
        
        let obj_ref = manager.allocate(test_obj.clone()).unwrap();
        
        assert!(!obj_ref.is_null());
        assert_eq!(obj_ref.heap_id(), 1); // Should be in default heap
        assert_eq!(obj_ref.object_id(), 1); // First object in heap
    }

    #[test]
    fn test_memory_manager_get_object() {
        let manager = MemoryManager::new();
        let test_obj = TestMemoryObject::new("Test Data".to_string(), 1);
        
        let obj_ref = manager.allocate(test_obj.clone()).unwrap();
        let retrieved = manager.get_object::<TestMemoryObject>(obj_ref).unwrap();
        
        assert_eq!(*retrieved, test_obj);
    }

    #[test]
    fn test_memory_manager_null_reference() {
        let manager = MemoryManager::new();
        let null_ref = ObjectReference::null();
        
        let result = manager.get_object::<TestMemoryObject>(null_ref);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MemoryError::InvalidReference(msg) => {
                assert!(msg.contains("Cannot dereference null"));
            },
            _ => panic!("Expected InvalidReference error"),
        }
    }

    #[test]
    fn test_memory_manager_type_mismatch() {
        let manager = MemoryManager::new();
        let test_obj = TestMemoryObject::new("Type test".to_string(), 1);
        
        let obj_ref = manager.allocate(test_obj).unwrap();
        
        // Try to retrieve as wrong type
        let result = manager.get_object::<NumberObject>(obj_ref);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MemoryError::InvalidReference(msg) => {
                assert!(msg.contains("Type mismatch"));
            },
            _ => panic!("Expected InvalidReference error for type mismatch"),
        }
    }

    #[test]
    fn test_memory_manager_multiple_heaps() {
        let manager = MemoryManager::new();
        let heap2_id = manager.create_heap().unwrap();
        
        // Allocate in default heap
        let obj1 = TestMemoryObject::new("Heap 1 object".to_string(), 1);
        let ref1 = manager.allocate(obj1.clone()).unwrap();
        
        // Switch to second heap and allocate
        manager.switch_heap(heap2_id).unwrap();
        let obj2 = TestMemoryObject::new("Heap 2 object".to_string(), 1);
        let ref2 = manager.allocate(obj2.clone()).unwrap();
        
        // Verify objects are in different heaps
        assert_eq!(ref1.heap_id(), 1);
        assert_eq!(ref2.heap_id(), heap2_id);
        
        // Should be able to retrieve from both heaps
        let retrieved1 = manager.get_object::<TestMemoryObject>(ref1).unwrap();
        let retrieved2 = manager.get_object::<TestMemoryObject>(ref2).unwrap();
        
        assert_eq!(*retrieved1, obj1);
        assert_eq!(*retrieved2, obj2);
    }

    #[test]
    fn test_memory_manager_invalid_heap_reference() {
        let manager = MemoryManager::new();
        
        // Create a reference to a non-existent heap
        let invalid_ref = ObjectReference::new(999, 1);
        
        let result = manager.get_object::<TestMemoryObject>(invalid_ref);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MemoryError::InvalidReference(msg) => {
                assert!(msg.contains("Heap 999 does not exist"));
            },
            _ => panic!("Expected InvalidReference error"),
        }
    }

    #[test]
    fn test_memory_manager_collect_heap() {
        let manager = MemoryManager::new();
        let heap2_id = manager.create_heap().unwrap();
        
        // Test GC on specific heap
        assert!(manager.collect_heap(1).is_ok());
        assert!(manager.collect_heap(heap2_id).is_ok());
        
        // Test GC on non-existent heap
        let result = manager.collect_heap(999);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MemoryError::HeapError(msg) => {
                assert!(msg.contains("Heap 999 does not exist"));
            },
            _ => panic!("Expected HeapError"),
        }
    }

    #[test]
    fn test_memory_manager_collect_all() {
        let manager = MemoryManager::new();
        manager.create_heap().unwrap();
        manager.create_heap().unwrap();
        
        // Should not fail even with multiple heaps
        assert!(manager.collect_all().is_ok());
    }

    #[test]
    fn test_memory_manager_allocation_sequence() {
        let manager = MemoryManager::new();
        
        let obj1 = TestMemoryObject::new("Object 1".to_string(), 1);
        let obj2 = TestMemoryObject::new("Object 2".to_string(), 1);
        let obj3 = NumberObject { value: 42 };
        
        let ref1 = manager.allocate(obj1.clone()).unwrap();
        let ref2 = manager.allocate(obj2.clone()).unwrap();
        let ref3 = manager.allocate(obj3.clone()).unwrap();
        
        // All should be in the same heap but different object IDs
        assert_eq!(ref1.heap_id(), 1);
        assert_eq!(ref2.heap_id(), 1);
        assert_eq!(ref3.heap_id(), 1);
        
        assert_eq!(ref1.object_id(), 1);
        assert_eq!(ref2.object_id(), 2);
        assert_eq!(ref3.object_id(), 3);
        
        // Verify retrieval
        assert_eq!(*manager.get_object::<TestMemoryObject>(ref1).unwrap(), obj1);
        assert_eq!(*manager.get_object::<TestMemoryObject>(ref2).unwrap(), obj2);
        assert_eq!(*manager.get_object::<NumberObject>(ref3).unwrap(), obj3);
    }

    #[test]
    fn test_memory_manager_heap_error_conversion() {
        let manager = MemoryManager::new();
        
        // Create an object reference to a non-existent object in an existing heap
        let invalid_ref = ObjectReference::new(1, 999);
        
        let result = manager.get_object::<TestMemoryObject>(invalid_ref);
        assert!(result.is_err());
        
        // The error should be converted from HeapError to MemoryError
        match result.unwrap_err() {
            MemoryError::InvalidReference(_) => {
                // Expected - the downcast failure gets converted to InvalidReference
            },
            MemoryError::HeapError(_) => {
                // Also acceptable - could be HeapError from get_object_any
            },
            _ => panic!("Expected MemoryError"),
        }
    }

    #[test]
    fn test_memory_manager_concurrent_access() {
        use std::thread;
        use std::sync::Arc;
        
        let manager = Arc::new(MemoryManager::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to allocate objects concurrently
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let obj = TestMemoryObject::new(format!("Thread {} object", i), 1);
                manager_clone.allocate(obj)
            });
            handles.push(handle);
        }
        
        let mut references = vec![];
        for handle in handles {
            let obj_ref = handle.join().unwrap().unwrap();
            references.push(obj_ref);
        }
        
        // All allocations should succeed and get unique object IDs
        assert_eq!(references.len(), 10);
        for (i, obj_ref) in references.iter().enumerate() {
            assert_eq!(obj_ref.heap_id(), 1);
            // Objects should have sequential IDs 1-10
            assert!(obj_ref.object_id() >= 1 && obj_ref.object_id() <= 10);
        }
        
        // All object IDs should be unique
        let mut object_ids: Vec<u64> = references.iter().map(|r| r.object_id()).collect();
        object_ids.sort();
        for i in 0..object_ids.len() - 1 {
            assert_ne!(object_ids[i], object_ids[i + 1], "Found duplicate object ID: {}", object_ids[i]);
        }
    }
}