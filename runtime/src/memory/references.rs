use std::fmt;

/// Object reference that uniquely identifies an object in any heap
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectReference {
    // First 32 bits: heap ID
    // Next 32 bits: object ID within that heap
    value: u64,
}

impl ObjectReference {
    /// Create a new object reference with the given heap ID and object ID
    pub fn new(heap_id: u32, object_id: u64) -> Self {
        let object_id = object_id & 0x00000000FFFFFFFF; // Ensure object_id fits in 32 bits
        let value = ((heap_id as u64) << 32) | object_id;
        Self { value }
    }
    
    /// Get the heap ID from this reference
    pub fn heap_id(&self) -> u32 {
        (self.value >> 32) as u32
    }
    
    /// Get the object ID within the heap
    pub fn object_id(&self) -> u64 {
        self.value & 0x00000000FFFFFFFF
    }
    
    /// Create a null reference
    pub fn null() -> Self {
        Self { value: 0 }
    }
    
    /// Check if this is a null reference
    pub fn is_null(&self) -> bool {
        self.value == 0
    }
}

impl fmt::Debug for ObjectReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            write!(f, "ObjectRef(null)")
        } else {
            write!(f, "ObjectRef(heap={}, id={})", self.heap_id(), self.object_id())
        }
    }
}

impl fmt::Display for ObjectReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            write!(f, "null")
        } else {
            write!(f, "Object@{}:{}", self.heap_id(), self.object_id())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_reference_creation() {
        let heap_id = 42;
        let object_id = 123;
        let obj_ref = ObjectReference::new(heap_id, object_id);
        
        assert_eq!(obj_ref.heap_id(), heap_id);
        assert_eq!(obj_ref.object_id(), object_id);
        assert!(!obj_ref.is_null());
    }

    #[test]
    fn test_object_reference_null() {
        let null_ref = ObjectReference::null();
        
        assert!(null_ref.is_null());
        assert_eq!(null_ref.heap_id(), 0);
        assert_eq!(null_ref.object_id(), 0);
    }

    #[test]
    fn test_object_reference_large_ids() {
        let heap_id = u32::MAX;
        let object_id = u32::MAX as u64;
        let obj_ref = ObjectReference::new(heap_id, object_id);
        
        assert_eq!(obj_ref.heap_id(), heap_id);
        assert_eq!(obj_ref.object_id(), object_id);
    }

    #[test]
    fn test_object_reference_object_id_truncation() {
        let heap_id = 1;
        let large_object_id = 0x123456789ABCDEF0; // Larger than 32 bits
        let obj_ref = ObjectReference::new(heap_id, large_object_id);
        
        // Object ID should be truncated to 32 bits
        assert_eq!(obj_ref.heap_id(), heap_id);
        assert_eq!(obj_ref.object_id(), large_object_id & 0x00000000FFFFFFFF);
    }

    #[test]
    fn test_object_reference_equality() {
        let ref1 = ObjectReference::new(1, 100);
        let ref2 = ObjectReference::new(1, 100);
        let ref3 = ObjectReference::new(1, 101);
        let ref4 = ObjectReference::new(2, 100);
        
        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
        assert_ne!(ref1, ref4);
    }

    #[test]
    fn test_object_reference_copy_clone() {
        let original = ObjectReference::new(5, 50);
        let copied = original; // Test Copy trait
        let cloned = original.clone(); // Test Clone trait
        
        assert_eq!(original, copied);
        assert_eq!(original, cloned);
        assert_eq!(copied, cloned);
    }

    #[test]
    fn test_object_reference_hash() {
        use std::collections::HashMap;
        
        let mut map = HashMap::new();
        let ref1 = ObjectReference::new(1, 10);
        let ref2 = ObjectReference::new(2, 20);
        
        map.insert(ref1, "Object 1".to_string());
        map.insert(ref2, "Object 2".to_string());
        
        assert_eq!(map.get(&ref1), Some(&"Object 1".to_string()));
        assert_eq!(map.get(&ref2), Some(&"Object 2".to_string()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_object_reference_debug_format() {
        let obj_ref = ObjectReference::new(42, 123);
        let debug_str = format!("{:?}", obj_ref);
        assert_eq!(debug_str, "ObjectRef(heap=42, id=123)");
        
        let null_ref = ObjectReference::null();
        let null_debug_str = format!("{:?}", null_ref);
        assert_eq!(null_debug_str, "ObjectRef(null)");
    }

    #[test]
    fn test_object_reference_display_format() {
        let obj_ref = ObjectReference::new(42, 123);
        let display_str = format!("{}", obj_ref);
        assert_eq!(display_str, "Object@42:123");
        
        let null_ref = ObjectReference::null();
        let null_display_str = format!("{}", null_ref);
        assert_eq!(null_display_str, "null");
    }

    #[test]
    fn test_object_reference_bit_operations() {
        let heap_id = 0x12345678;
        let object_id = 0x87654321;
        let obj_ref = ObjectReference::new(heap_id, object_id);
        
        // Verify internal bit representation
        let expected_value = ((heap_id as u64) << 32) | object_id;
        assert_eq!(obj_ref.value, expected_value);
        
        // Verify extraction
        assert_eq!(obj_ref.heap_id(), heap_id);
        assert_eq!(obj_ref.object_id(), object_id);
    }

    #[test]
    fn test_object_reference_zero_ids() {
        let obj_ref = ObjectReference::new(0, 0);
        
        // This should be equivalent to null
        assert!(obj_ref.is_null());
        assert_eq!(obj_ref.heap_id(), 0);
        assert_eq!(obj_ref.object_id(), 0);
        assert_eq!(obj_ref, ObjectReference::null());
    }

    #[test]
    fn test_object_reference_max_heap_id() {
        let max_heap_id = u32::MAX;
        let object_id = 1;
        let obj_ref = ObjectReference::new(max_heap_id, object_id);
        
        assert_eq!(obj_ref.heap_id(), max_heap_id);
        assert_eq!(obj_ref.object_id(), object_id);
        assert!(!obj_ref.is_null());
    }

    #[test]
    fn test_object_reference_different_heap_same_object() {
        let obj_ref1 = ObjectReference::new(1, 123);
        let obj_ref2 = ObjectReference::new(2, 123);
        
        // Same object ID but different heap should be different references
        assert_ne!(obj_ref1, obj_ref2);
        assert_eq!(obj_ref1.object_id(), obj_ref2.object_id());
        assert_ne!(obj_ref1.heap_id(), obj_ref2.heap_id());
    }
}