use loaf::{Runtime, VERSION};
use loaf::runtime::RuntimeConfig;
use loaf::utils::{generate_demo_module, write_bytecode};
use std::path::Path;

fn main() {
    println!("Loaf Bytecode Runtime v{}", VERSION);
    
    // Create demo bytecode file
    let demo_path = Path::new("demo.crouton");
    let demo_module = generate_demo_module();
    match write_bytecode(&demo_module, demo_path) {
        Ok(_) => println!("Created demo bytecode file: demo.crouton"),
        Err(e) => {
            eprintln!("Failed to create demo crouton file: {}", e);
            return;
        }
    }
    
    // Initialize the runtime with debug mode
    let runtime = Runtime::with_config(
        RuntimeConfig::default()
            .with_debug_mode(true)
            .with_stack_trace(true)  // Enable stack tracing
    ).expect("Failed to initialize the runtime");
    
    // Execute the demo file
    println!("\nExecuting demo crouton...");
    match runtime.execute_file("demo.crouton") {
        Ok(result) => println!("Execution completed with result: {}", result),
        Err(e) => eprintln!("Execution failed: {}", e),
    }
    
    // Show heap information
    let _default_heap_id = 1; // Default heap ID (prefixed with underscore to avoid warning)
    println!("\nHeap information:");
    println!("Current heap ID: {}", runtime.current_heap_id());
    
    // Create additional heaps
    match runtime.create_heap() {
        Ok(heap_id) => println!("Created new heap with ID: {}", heap_id),
        Err(e) => eprintln!("Failed to create heap: {}", e),
    }
    
    // Try collecting garbage
    println!("\nTriggering garbage collection...");
    if let Err(e) = runtime.collect_all() {
        eprintln!("GC failed: {}", e);
    } else {
        println!("GC completed successfully");
    }
}
