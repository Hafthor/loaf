mod manager;
mod standard_heap;
mod mark_sweep;

pub use manager::HeapManager;
pub use standard_heap::StandardHeap;

use std::sync::Arc;
use std::any::Any;
use thiserror::Error;
use crate::memory::MemoryObject;

/// Errors that can occur in heap operations
#[derive(Error, Debug)]
pub enum HeapError {
    #[error("Heap allocation failed: {0}")]
    AllocationFailed(String),
    
    #[error("Invalid object ID: {0}")]
    InvalidObjectId(String),
    
    #[error("Object type mismatch")]
    TypeMismatch,
    
    #[error("Garbage collection failed: {0}")]
    GCFailed(String),
}

/// Result type for heap operations
pub type HeapResult<T> = Result<T, HeapError>;

/// The Heap trait defines the operations a heap implementation must support
/// Now made object-safe by removing generic methods
pub trait Heap: Send + Sync {
    /// Allocate an object in this heap and return its object ID
    fn allocate_object(&self, object: Box<dyn Any + Send + Sync>, size: usize, type_id: u32) -> HeapResult<u64>;
    
    /// Get a reference to an object by its ID as Any
    fn get_object_any(&self, object_id: u64) -> HeapResult<Arc<dyn Any + Send + Sync>>;
    
    /// Trigger garbage collection
    fn collect(&self) -> HeapResult<()>;
    
    /// Get the current number of objects in this heap
    fn object_count(&self) -> usize;
    
    /// Get the total memory usage of this heap in bytes
    fn memory_usage(&self) -> usize;
    
    /// Get the heap's unique ID
    fn id(&self) -> u32;
}

/// Extension methods for Heap trait to provide a more type-safe API
pub trait HeapExt {
    /// Allocate an object in this heap and return its object ID with type safety
    fn allocate<T: MemoryObject + 'static>(&self, object: T) -> HeapResult<u64>;
    
    /// Get a reference to an object by its ID with type safety
    fn get_object<T: MemoryObject + 'static>(&self, object_id: u64) -> HeapResult<Arc<T>>;
}

impl<H: ?Sized + Heap> HeapExt for H {
    fn allocate<T: MemoryObject + 'static>(&self, object: T) -> HeapResult<u64> {
        let size = object.size();
        let type_id = object.type_id();
        self.allocate_object(Box::new(object), size, type_id)
    }
    
    fn get_object<T: MemoryObject + 'static>(&self, object_id: u64) -> HeapResult<Arc<T>> {
        let any_ref = self.get_object_any(object_id)?;
        let downcast = Arc::downcast::<T>(any_ref);
        
        match downcast {
            Ok(typed_ref) => Ok(typed_ref),
            Err(_) => Err(HeapError::TypeMismatch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryObject;
    use std::sync::Arc;

    // Test struct implementing MemoryObject for heap tests
    #[derive(Debug, Clone, PartialEq)]
    struct TestMemoryObject {
        data: String,
        object_size: usize,
        object_type_id: u32,
    }

    impl TestMemoryObject {
        fn new(data: String, type_id: u32) -> Self {
            let size = data.len() + std::mem::size_of::<String>() + 8;
            Self {
                data,
                object_size: size,
                object_type_id: type_id,
            }
        }
    }

    impl MemoryObject for TestMemoryObject {
        fn size(&self) -> usize {
            self.object_size
        }

        fn type_id(&self) -> u32 {
            self.object_type_id
        }
    }

    #[test]
    fn test_heap_error_display() {
        let allocation_error = HeapError::AllocationFailed("Out of memory".to_string());
        let invalid_id_error = HeapError::InvalidObjectId("ID 123 not found".to_string());
        let type_mismatch_error = HeapError::TypeMismatch;
        let gc_error = HeapError::GCFailed("Mark phase failed".to_string());

        assert_eq!(allocation_error.to_string(), "Heap allocation failed: Out of memory");
        assert_eq!(invalid_id_error.to_string(), "Invalid object ID: ID 123 not found");
        assert_eq!(type_mismatch_error.to_string(), "Object type mismatch");
        assert_eq!(gc_error.to_string(), "Garbage collection failed: Mark phase failed");
    }

    #[test]
    fn test_heap_trait_basic_operations() {
        let heap = StandardHeap::new(1).unwrap();
        let heap_ref: &dyn Heap = &heap;

        // Test basic heap properties
        assert_eq!(heap_ref.id(), 1);
        assert_eq!(heap_ref.object_count(), 0);
        assert_eq!(heap_ref.memory_usage(), 0);

        // Test GC operation (should not fail even if not implemented)
        assert!(heap_ref.collect().is_ok());
    }

    #[test]
    fn test_heap_trait_object_allocation() {
        let heap = StandardHeap::new(2).unwrap();
        let test_obj = TestMemoryObject::new("Heap trait test".to_string(), 1);
        let size = MemoryObject::size(&test_obj);

        // Allocate using the low-level Heap trait method
        let object_id = heap.allocate_object(Box::new(test_obj.clone()), size, MemoryObject::type_id(&test_obj)).unwrap();

        assert_eq!(object_id, 1);
        assert_eq!(heap.object_count(), 1);
        assert_eq!(heap.memory_usage(), size);

        // Retrieve using the low-level method
        let any_ref = heap.get_object_any(object_id).unwrap();
        let downcast_ref = any_ref.downcast::<TestMemoryObject>().unwrap();
        assert_eq!(*downcast_ref, test_obj);
    }

    #[test]
    fn test_heap_ext_trait_typed_operations() {
        let heap = StandardHeap::new(3).unwrap();
        let test_obj = TestMemoryObject::new("HeapExt test".to_string(), 2);

        // Use the high-level HeapExt methods
        let object_id = heap.allocate(test_obj.clone()).unwrap();
        let retrieved = heap.get_object::<TestMemoryObject>(object_id).unwrap();

        assert_eq!(*retrieved, test_obj);
        assert_eq!(heap.object_count(), 1);
    }

    #[test]
    fn test_heap_ext_trait_type_safety() {
        #[derive(Debug, Clone, PartialEq)]
        struct AnotherTestObject {
            value: i32,
        }

        impl MemoryObject for AnotherTestObject {
            fn size(&self) -> usize {
                std::mem::size_of::<i32>()
            }

            fn type_id(&self) -> u32 {
                99
            }
        }

        let heap = StandardHeap::new(4).unwrap();
        let test_obj = TestMemoryObject::new("Type safety test".to_string(), 1);

        let object_id = heap.allocate(test_obj).unwrap();

        // Try to retrieve with wrong type - should fail with TypeMismatch
        let wrong_type_result = heap.get_object::<AnotherTestObject>(object_id);
        assert!(wrong_type_result.is_err());

        match wrong_type_result.unwrap_err() {
            HeapError::TypeMismatch => {},
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_heap_as_trait_object() {
        let heap = StandardHeap::new(5).unwrap();
        let heap_trait: Arc<dyn Heap> = Arc::new(heap);

        // Test that we can use heap through trait object
        assert_eq!(heap_trait.id(), 5);
        assert_eq!(heap_trait.object_count(), 0);
        assert_eq!(heap_trait.memory_usage(), 0);

        let test_obj = TestMemoryObject::new("Trait object test".to_string(), 1);
        let size = MemoryObject::size(&test_obj);
        let type_id = MemoryObject::type_id(&test_obj);

        let object_id = heap_trait.allocate_object(Box::new(test_obj.clone()), size, type_id).unwrap();
        assert_eq!(object_id, 1);
        assert_eq!(heap_trait.object_count(), 1);

        let retrieved_any = heap_trait.get_object_any(object_id).unwrap();
        let retrieved = retrieved_any.downcast::<TestMemoryObject>().unwrap();
        assert_eq!(*retrieved, test_obj);
    }

    #[test]
    fn test_heap_ext_with_trait_object() {
        use crate::heap::HeapExt;

        let heap = StandardHeap::new(6).unwrap();
        let heap_ref: &dyn Heap = &heap;

        let test_obj = TestMemoryObject::new("HeapExt with trait object".to_string(), 1);

        // Use HeapExt methods on trait object reference
        let object_id = heap_ref.allocate(test_obj.clone()).unwrap();
        let retrieved = heap_ref.get_object::<TestMemoryObject>(object_id).unwrap();

        assert_eq!(*retrieved, test_obj);
    }

    #[test]
    fn test_multiple_heap_types_compatibility() {
        // Test that different heap instances are compatible as trait objects
        let heap1 = Arc::new(StandardHeap::new(1).unwrap()) as Arc<dyn Heap>;
        let heap2 = Arc::new(StandardHeap::new(2).unwrap()) as Arc<dyn Heap>;

        let obj1 = TestMemoryObject::new("Heap 1 object".to_string(), 1);
        let obj2 = TestMemoryObject::new("Heap 2 object".to_string(), 1);

        // Each heap should work independently
        let id1 = heap1.allocate(obj1.clone()).unwrap();
        let id2 = heap2.allocate(obj2.clone()).unwrap();

        assert_eq!(heap1.id(), 1);
        assert_eq!(heap2.id(), 2);
        assert_eq!(heap1.object_count(), 1);
        assert_eq!(heap2.object_count(), 1);

        let retrieved1 = heap1.get_object::<TestMemoryObject>(id1).unwrap();
        let retrieved2 = heap2.get_object::<TestMemoryObject>(id2).unwrap();

        assert_eq!(*retrieved1, obj1);
        assert_eq!(*retrieved2, obj2);
    }

    #[test]
    fn test_heap_result_type() {
        // Test that HeapResult works correctly
        let success: HeapResult<u64> = Ok(42);
        let error: HeapResult<u64> = Err(HeapError::AllocationFailed("Test error".to_string()));

        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        assert!(error.is_err());
        match error.unwrap_err() {
            HeapError::AllocationFailed(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected AllocationFailed error"),
        }
    }

    #[test]
    fn test_heap_error_debug_format() {
        let error = HeapError::TypeMismatch;
        let debug_string = format!("{:?}", error);
        assert_eq!(debug_string, "TypeMismatch");

        let error2 = HeapError::InvalidObjectId("test".to_string());
        let debug_string2 = format!("{:?}", error2);
        assert!(debug_string2.contains("InvalidObjectId"));
        assert!(debug_string2.contains("test"));
    }
}