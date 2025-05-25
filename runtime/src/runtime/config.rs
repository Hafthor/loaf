//! Runtime configuration options

/// Configuration options for the Loaf runtime
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub debug_mode: bool,
    pub stack_trace: bool,
    pub gc_threshold: usize,
    pub gc_enabled: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            stack_trace: false,
            gc_threshold: 10000,
            gc_enabled: true,
        }
    }
}

impl RuntimeConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enable or disable debug mode
    pub fn with_debug_mode(mut self, debug_mode: bool) -> Self {
        self.debug_mode = debug_mode;
        self
    }
    
    /// Enable or disable stack trace
    pub fn with_stack_trace(mut self, stack_trace: bool) -> Self {
        self.stack_trace = stack_trace;
        self
    }
    
    /// Set the garbage collection threshold
    pub fn with_gc_threshold(mut self, threshold: usize) -> Self {
        self.gc_threshold = threshold;
        self
    }
    
    /// Enable or disable garbage collection
    pub fn with_gc_enabled(mut self, enabled: bool) -> Self {
        self.gc_enabled = enabled;
        self
    }
}