use std::collections::HashSet;
use crate::heap::HeapResult;
use crate::memory::ObjectReference;

/// Mark-Sweep garbage collector implementation
pub struct MarkSweepGC {
    // Root objects that are directly accessible
    roots: HashSet<ObjectReference>,
}

impl MarkSweepGC {
    pub fn new() -> Self {
        Self {
            roots: HashSet::new(),
        }
    }
    
    /// Add a root reference
    pub fn add_root(&mut self, reference: ObjectReference) {
        if !reference.is_null() {
            self.roots.insert(reference);
        }
    }
    
    /// Remove a root reference
    pub fn remove_root(&mut self, reference: &ObjectReference) {
        self.roots.remove(reference);
    }
    
    /// Clear all roots
    pub fn clear_roots(&mut self) {
        self.roots.clear();
    }
    
    /// Mark phase: mark all reachable objects
    pub fn mark(&self, visit_object: impl Fn(&ObjectReference) -> HeapResult<Vec<ObjectReference>>) -> HeapResult<HashSet<ObjectReference>> {
        let mut marked = HashSet::new();
        let mut stack = Vec::new();
        
        // Start with roots
        for root in &self.roots {
            stack.push(*root);
        }
        
        // Process stack until empty
        while let Some(reference) = stack.pop() {
            if marked.contains(&reference) {
                continue;
            }
            
            // Mark this object
            marked.insert(reference);
            
            // Get all references from this object and add to stack
            let references = visit_object(&reference)?;
            for child_ref in references {
                if !child_ref.is_null() && !marked.contains(&child_ref) {
                    stack.push(child_ref);
                }
            }
        }
        
        Ok(marked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::HeapError;

    #[test]
    fn test_mark_sweep_gc_creation() {
        let gc = MarkSweepGC::new();
        assert_eq!(gc.roots.len(), 0);
    }

    #[test]
    fn test_mark_sweep_gc_add_root() {
        let mut gc = MarkSweepGC::new();
        let ref1 = ObjectReference::new(1, 100);
        let ref2 = ObjectReference::new(1, 200);
        
        gc.add_root(ref1);
        gc.add_root(ref2);
        
        assert_eq!(gc.roots.len(), 2);
        assert!(gc.roots.contains(&ref1));
        assert!(gc.roots.contains(&ref2));
    }

    #[test]
    fn test_mark_sweep_gc_add_null_root() {
        let mut gc = MarkSweepGC::new();
        let null_ref = ObjectReference::null();
        
        gc.add_root(null_ref);
        
        // Null references should not be added
        assert_eq!(gc.roots.len(), 0);
    }

    #[test]
    fn test_mark_sweep_gc_add_duplicate_root() {
        let mut gc = MarkSweepGC::new();
        let ref1 = ObjectReference::new(1, 100);
        
        gc.add_root(ref1);
        gc.add_root(ref1); // Add same reference again
        
        // Should only be one reference in the set
        assert_eq!(gc.roots.len(), 1);
        assert!(gc.roots.contains(&ref1));
    }

    #[test]
    fn test_mark_sweep_gc_remove_root() {
        let mut gc = MarkSweepGC::new();
        let ref1 = ObjectReference::new(1, 100);
        let ref2 = ObjectReference::new(1, 200);
        
        gc.add_root(ref1);
        gc.add_root(ref2);
        assert_eq!(gc.roots.len(), 2);
        
        gc.remove_root(&ref1);
        assert_eq!(gc.roots.len(), 1);
        assert!(!gc.roots.contains(&ref1));
        assert!(gc.roots.contains(&ref2));
    }

    #[test]
    fn test_mark_sweep_gc_remove_nonexistent_root() {
        let mut gc = MarkSweepGC::new();
        let ref1 = ObjectReference::new(1, 100);
        let ref2 = ObjectReference::new(1, 200);
        
        gc.add_root(ref1);
        gc.remove_root(&ref2); // Remove reference that wasn't added
        
        // Should still have the original reference
        assert_eq!(gc.roots.len(), 1);
        assert!(gc.roots.contains(&ref1));
    }

    #[test]
    fn test_mark_sweep_gc_clear_roots() {
        let mut gc = MarkSweepGC::new();
        let ref1 = ObjectReference::new(1, 100);
        let ref2 = ObjectReference::new(1, 200);
        let ref3 = ObjectReference::new(2, 300);
        
        gc.add_root(ref1);
        gc.add_root(ref2);
        gc.add_root(ref3);
        assert_eq!(gc.roots.len(), 3);
        
        gc.clear_roots();
        assert_eq!(gc.roots.len(), 0);
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_no_roots() {
        let gc = MarkSweepGC::new();
        
        // Visit function that should never be called since there are no roots
        let visit_fn = |_: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            panic!("Visit function should not be called with no roots");
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 0);
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_single_root() {
        let mut gc = MarkSweepGC::new();
        let root_ref = ObjectReference::new(1, 100);
        gc.add_root(root_ref);
        
        // Visit function that returns no child references
        let visit_fn = |_: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            Ok(vec![])
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 1);
        assert!(marked.contains(&root_ref));
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_with_children() {
        let mut gc = MarkSweepGC::new();
        let root_ref = ObjectReference::new(1, 100);
        let child1_ref = ObjectReference::new(1, 200);
        let child2_ref = ObjectReference::new(1, 300);
        
        gc.add_root(root_ref);
        
        // Visit function that returns child references for the root
        let visit_fn = |obj_ref: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            if *obj_ref == root_ref {
                Ok(vec![child1_ref, child2_ref])
            } else {
                Ok(vec![])
            }
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 3);
        assert!(marked.contains(&root_ref));
        assert!(marked.contains(&child1_ref));
        assert!(marked.contains(&child2_ref));
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_cyclic_references() {
        let mut gc = MarkSweepGC::new();
        let ref1 = ObjectReference::new(1, 100);
        let ref2 = ObjectReference::new(1, 200);
        
        gc.add_root(ref1);
        
        // Create a cycle: ref1 -> ref2 -> ref1
        let visit_fn = |obj_ref: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            if *obj_ref == ref1 {
                Ok(vec![ref2])
            } else if *obj_ref == ref2 {
                Ok(vec![ref1])
            } else {
                Ok(vec![])
            }
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 2);
        assert!(marked.contains(&ref1));
        assert!(marked.contains(&ref2));
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_null_children() {
        let mut gc = MarkSweepGC::new();
        let root_ref = ObjectReference::new(1, 100);
        let null_ref = ObjectReference::null();
        let valid_ref = ObjectReference::new(1, 200);
        
        gc.add_root(root_ref);
        
        // Visit function that returns a mix of null and valid references
        let visit_fn = |obj_ref: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            if *obj_ref == root_ref {
                Ok(vec![null_ref, valid_ref])
            } else {
                Ok(vec![])
            }
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 2);
        assert!(marked.contains(&root_ref));
        assert!(marked.contains(&valid_ref));
        assert!(!marked.contains(&null_ref));
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_multiple_roots() {
        let mut gc = MarkSweepGC::new();
        let root1 = ObjectReference::new(1, 100);
        let root2 = ObjectReference::new(1, 200);
        let shared_child = ObjectReference::new(1, 300);
        let root1_child = ObjectReference::new(1, 400);
        let root2_child = ObjectReference::new(1, 500);
        
        gc.add_root(root1);
        gc.add_root(root2);
        
        // Visit function where both roots point to a shared child plus their own children
        let visit_fn = |obj_ref: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            if *obj_ref == root1 {
                Ok(vec![shared_child, root1_child])
            } else if *obj_ref == root2 {
                Ok(vec![shared_child, root2_child])
            } else {
                Ok(vec![])
            }
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 5);
        assert!(marked.contains(&root1));
        assert!(marked.contains(&root2));
        assert!(marked.contains(&shared_child));
        assert!(marked.contains(&root1_child));
        assert!(marked.contains(&root2_child));
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_visit_error() {
        let mut gc = MarkSweepGC::new();
        let root_ref = ObjectReference::new(1, 100);
        gc.add_root(root_ref);
        
        // Visit function that returns an error
        let visit_fn = |_: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            Err(HeapError::InvalidObjectId("Test error".to_string()))
        };
        
        let result = gc.mark(visit_fn);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            HeapError::InvalidObjectId(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected InvalidObjectId error"),
        }
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_deep_object_graph() {
        let mut gc = MarkSweepGC::new();
        let root = ObjectReference::new(1, 100);
        let obj1 = ObjectReference::new(1, 200);
        let obj2 = ObjectReference::new(1, 300);
        let obj3 = ObjectReference::new(1, 400);
        let leaf = ObjectReference::new(1, 500);
        
        gc.add_root(root);
        
        // Create a chain: root -> obj1 -> obj2 -> obj3 -> leaf
        let visit_fn = |obj_ref: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            match *obj_ref {
                ref r if r == &root => Ok(vec![obj1]),
                ref r if r == &obj1 => Ok(vec![obj2]),
                ref r if r == &obj2 => Ok(vec![obj3]),
                ref r if r == &obj3 => Ok(vec![leaf]),
                _ => Ok(vec![]),
            }
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 5);
        assert!(marked.contains(&root));
        assert!(marked.contains(&obj1));
        assert!(marked.contains(&obj2));
        assert!(marked.contains(&obj3));
        assert!(marked.contains(&leaf));
    }

    #[test]
    fn test_mark_sweep_gc_mark_phase_already_marked_skip() {
        let mut gc = MarkSweepGC::new();
        let root = ObjectReference::new(1, 100);
        let obj1 = ObjectReference::new(1, 200);
        let obj2 = ObjectReference::new(1, 300);
        
        gc.add_root(root);
        
        let visit_count = std::cell::RefCell::new(std::collections::HashMap::new());
        
        // Create a diamond pattern: root -> obj1, obj2; obj1 -> obj2; obj2 -> nothing
        let visit_fn = |obj_ref: &ObjectReference| -> HeapResult<Vec<ObjectReference>> {
            // Count visits to ensure objects aren't visited multiple times
            *visit_count.borrow_mut().entry(*obj_ref).or_insert(0) += 1;
            
            match *obj_ref {
                ref r if r == &root => Ok(vec![obj1, obj2]),
                ref r if r == &obj1 => Ok(vec![obj2]),
                _ => Ok(vec![]),
            }
        };
        
        let marked = gc.mark(visit_fn).unwrap();
        assert_eq!(marked.len(), 3);
        assert!(marked.contains(&root));
        assert!(marked.contains(&obj1));
        assert!(marked.contains(&obj2));
        
        // Verify obj2 was only visited once (not twice) due to early termination
        let counts = visit_count.borrow();
        assert_eq!(*counts.get(&root).unwrap(), 1);
        assert_eq!(*counts.get(&obj1).unwrap(), 1);
        assert_eq!(*counts.get(&obj2).unwrap(), 1);
    }
}