# Crouton - The loaf Bytecode Specification

## Overview

The loaf bytecode is a stack-based virtual machine instruction set designed for the loaf programming language. It provides efficient execution of loaf programs through a compact binary representation with support for advanced features like structured exception handling, heap management, and promise-based concurrency.

## File Format

### Header Structure

Every loaf bytecode file begins with a standard header:

```
Offset | Size | Type   | Description
-------|------|--------|---------------------------
0x00   | 4    | u32    | Magic number (0x4C4F4146 - "LOAF")
0x04   | 1    | u8     | Major version (currently 1)
0x05   | 1    | u8     | Minor version  
0x06   | 2    | u16    | Patch version
0x08   | 4    | u32    | Module name length
0x0C   | N    | bytes  | Module name (UTF-8)
```

### Constants Section

Following the header is the constants section containing literal values:

```
Offset | Size | Type   | Description
-------|------|--------|---------------------------
0x00   | 4    | u32    | Number of constants
0x04   | ...  | entry  | Constant entries
```

Each constant entry has the format:

```
Offset | Size | Type   | Description
-------|------|--------|---------------------------
0x00   | 1    | u8     | Constant type (see below)
0x01   | ...  | data   | Constant data
```

**Constant Types:**
- `0x00`: Null (no additional data)
- `0x01`: Integer (8 bytes, i64, big-endian)
- `0x02`: Float (8 bytes, f64, big-endian, IEEE 754)
- `0x03`: String (4-byte length prefix + UTF-8 data)
- `0x04`: Boolean (1 byte: 0x00 = false, 0x01 = true)

### Instructions Section

The instructions section contains the executable bytecode:

```
Offset | Size | Type   | Description
-------|------|--------|---------------------------
0x00   | 4    | u32    | Number of instructions
0x04   | ...  | instr  | Instruction entries
```

Each instruction consists of:
- 1 byte opcode
- 0-3 operands (4 bytes each, u32, big-endian)

## Instruction Set Architecture

### Stack Machine Model

The loaf VM uses a stack-based execution model with:
- **Evaluation Stack**: Primary data stack for computations
- **Local Variables**: Indexed storage for function-local data
- **Heap Management**: Multiple garbage-collected heaps
- **Exception Handling**: Structured try/catch/finally blocks

### Data Types

The VM supports the following value types:
- `Null`: Absence of value
- `Integer`: 64-bit signed integers
- `Float`: 64-bit IEEE 754 floating-point
- `Boolean`: True/false values
- `String`: UTF-8 encoded text
- `Object`: Heap-allocated objects with reference counting
- `Array`: Dynamic arrays of values
- `HeapId`: Heap identifier for heap management
- `ProgramCounter`: Code address for control flow
- `Exception`: Structured exception data with stack traces

## Instruction Reference

### Control Operations (0x00-0x0F)

#### NOP (0x00)
**Operands:** None  
**Stack Effect:** No change  
**Description:** No operation. Used for padding or debugging.

#### HALT (0x01)
**Operands:** None  
**Stack Effect:** No change  
**Description:** Terminates program execution immediately.

#### PRINT (0x02)
**Operands:** None  
**Stack Effect:** `[value] -> []`  
**Description:** Pops the top value from the stack and prints it to stdout.

### Stack Manipulation (0x10-0x1F)

#### PUSH (0x10)
**Operands:** `constant_index` (u32)  
**Stack Effect:** `[] -> [value]`  
**Description:** Pushes a constant from the constants table onto the stack.

#### POP (0x11)
**Operands:** None  
**Stack Effect:** `[value] -> []`  
**Description:** Removes the top value from the stack.

#### DUP (0x12)
**Operands:** None  
**Stack Effect:** `[value] -> [value, value]`  
**Description:** Duplicates the top value on the stack.

#### SWAP (0x13)
**Operands:** None  
**Stack Effect:** `[a, b] -> [b, a]`  
**Description:** Swaps the top two values on the stack.

### Arithmetic Operations (0x20-0x2F)

#### ADD (0x20)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Adds two numbers. Supports integer and float addition with type promotion.

#### SUB (0x21)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Subtracts `b` from `a` (i.e., computes `a - b`).

#### MUL (0x22)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Multiplies two numbers.

#### DIV (0x23)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Divides `a` by `b` (i.e., computes `a / b`). Throws exception on division by zero.

#### NEG (0x24)
**Operands:** None  
**Stack Effect:** `[value] -> [result]`  
**Description:** Negates a number (computes `-value`).

### Bitwise Operations (0x30-0x3F)

#### BITAND (0x30)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Performs bitwise AND operation on two integers.

#### BITOR (0x31)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Performs bitwise OR operation on two integers.

#### BITXOR (0x32)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Performs bitwise XOR operation on two integers.

#### BITNOT (0x33)
**Operands:** None  
**Stack Effect:** `[value] -> [result]`  
**Description:** Performs bitwise NOT operation on an integer.

#### SHIFTLEFT (0x34)
**Operands:** None  
**Stack Effect:** `[value, shift] -> [result]`  
**Description:** Shifts `value` left by `shift` bits.

#### SHIFTRIGHT (0x35)
**Operands:** None  
**Stack Effect:** `[value, shift] -> [result]`  
**Description:** Shifts `value` right by `shift` bits (arithmetic shift).

#### ROTATELEFT (0x36)
**Operands:** None  
**Stack Effect:** `[value, rotate] -> [result]`  
**Description:** Rotates `value` left by `rotate` bits.

#### ROTATERIGHT (0x37)
**Operands:** None  
**Stack Effect:** `[value, rotate] -> [result]`  
**Description:** Rotates `value` right by `rotate` bits.

### Logical Operations (0x40-0x4F)

#### AND (0x40)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Performs logical AND. Returns true if both values are truthy.

#### OR (0x41)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Performs logical OR. Returns true if either value is truthy.

#### NOT (0x42)
**Operands:** None  
**Stack Effect:** `[value] -> [result]`  
**Description:** Performs logical NOT. Returns the boolean negation of the value.

### Comparison Operations (0x50-0x5F)

#### EQ (0x50)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Tests equality. Returns true if values are equal.

#### NEQ (0x51)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Tests inequality. Returns true if values are not equal.

#### LT (0x52)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Tests if `a < b`.

#### LTE (0x53)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Tests if `a <= b`.

#### GT (0x54)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Tests if `a > b`.

#### GTE (0x55)
**Operands:** None  
**Stack Effect:** `[a, b] -> [result]`  
**Description:** Tests if `a >= b`.

### Control Flow (0x60-0x6F)

#### JUMP (0x60)
**Operands:** `address` (u32)  
**Stack Effect:** No change  
**Description:** Unconditional jump to the specified address.

#### JUMPIF (0x61)
**Operands:** `address` (u32)  
**Stack Effect:** `[condition] -> []`  
**Description:** Jumps to address if the condition is truthy.

#### JUMPIFNOT (0x62)
**Operands:** `address` (u32)  
**Stack Effect:** `[condition] -> []`  
**Description:** Jumps to address if the condition is falsy.

#### CALL (0x63)
**Operands:** `address` (u32)  
**Stack Effect:** Varies by function  
**Description:** Calls a function at the specified address. Pushes return address onto call stack.

#### RETURN (0x64)
**Operands:** None  
**Stack Effect:** Varies by function  
**Description:** Returns from a function call. Pops return address from call stack.

### Exception Handling (0x6A-0x6F)

#### TRYBLOCK (0x6A)
**Operands:** `catch_address` (u32), `finally_address` (u32), `end_address` (u32)  
**Stack Effect:** No change  
**Description:** Establishes a try block with catch and finally handlers. Sets up exception handling context.

#### CATCHBLOCK (0x6B)
**Operands:** None  
**Stack Effect:** `[] -> [exception]` (when exception caught)  
**Description:** Marks the beginning of a catch block. The caught exception is pushed onto the stack.

#### FINALLYBLOCK (0x6C)
**Operands:** None  
**Stack Effect:** No change  
**Description:** Marks the beginning of a finally block. Always executed regardless of exceptions.

#### ENDTRY (0x6D)
**Operands:** None  
**Stack Effect:** No change  
**Description:** Marks the end of a try/catch/finally construct. Cleans up exception handling context.

#### THROW (0x6E)
**Operands:** None  
**Stack Effect:** `[exception] -> []`  
**Description:** Throws an exception. Begins unwinding the stack to find appropriate handler.

#### RETHROW (0x6F)
**Operands:** None  
**Stack Effect:** No change  
**Description:** Re-throws the current exception during catch block processing.

### Local Variables (0x70-0x7F)

#### STORELOCAL (0x70)
**Operands:** `slot` (u32)  
**Stack Effect:** `[value] -> []`  
**Description:** Stores the top stack value in the specified local variable slot.

#### LOADLOCAL (0x71)
**Operands:** `slot` (u32)  
**Stack Effect:** `[] -> [value]`  
**Description:** Loads a value from the specified local variable slot onto the stack.

### Heap Operations (0x80-0x8F)

#### CREATEHEAP (0x80)
**Operands:** None  
**Stack Effect:** `[] -> [heap_id]`  
**Description:** Creates a new isolated heap and pushes its ID onto the stack.

#### SWITCHHEAP (0x81)
**Operands:** None  
**Stack Effect:** `[heap_id] -> []`  
**Description:** Switches the current execution context to use the specified heap.

#### COLLECTHEAP (0x82)
**Operands:** `heap_id` (u32)  
**Stack Effect:** No change  
**Description:** Forces garbage collection on the specified heap.

### Array Operations (0x90-0x9F)

#### NEWARRAY (0x90)
**Operands:** `size` (u32)  
**Stack Effect:** `[elem1, elem2, ..., elemN] -> [array]`  
**Description:** Creates a new array with `size` elements popped from the stack.

#### GETELEMENT (0x91)
**Operands:** None  
**Stack Effect:** `[array, index] -> [value]`  
**Description:** Gets an element from an array at the specified index.

#### SETELEMENT (0x92)
**Operands:** None  
**Stack Effect:** `[array, index, value] -> []`  
**Description:** Sets an element in an array at the specified index.

#### ARRAYLENGTH (0x93)
**Operands:** None  
**Stack Effect:** `[array] -> [length]`  
**Description:** Gets the length of an array.

## Exception Handling Model

The loaf VM implements structured exception handling with try/catch/finally semantics:

### Exception Flow

1. **TryBlock Setup**: Establishes exception handling context with catch/finally addresses
2. **Normal Execution**: Code executes normally within the try block
3. **Exception Thrown**: When an exception occurs, the VM unwinds the stack
4. **Handler Search**: The VM searches for the nearest try block with appropriate handlers
5. **Catch Execution**: If a catch block exists, the exception is pushed onto the stack and execution continues
6. **Finally Execution**: Finally blocks always execute, regardless of exceptions
7. **Cleanup**: Exception handling context is cleaned up with EndTry

### Exception Data Structure

Exceptions carry structured information:
- **Type**: String identifier for the exception type
- **Message**: Human-readable error description  
- **Stack Trace**: Array of stack frames showing execution path

### Exception Types

Common exception types include:
- `"DivisionByZero"`: Arithmetic division by zero
- `"IndexOutOfBounds"`: Array access beyond bounds
- `"NullPointerException"`: Access to null object
- `"TypeError"`: Type mismatch in operations
- `"StackOverflow"`: Execution stack exhaustion
- `"HeapExhaustion"`: Memory allocation failure

## Memory Management

### Heap Architecture

The loaf VM supports multiple isolated heaps:
- Each heap maintains its own object space
- Heaps can be created, switched, and garbage collected independently
- Objects cannot reference across heap boundaries
- Enables isolation for security and performance

### Garbage Collection

- **Mark-and-Sweep**: Primary collection algorithm
- **Reference Counting**: For immediate cleanup of cycles
- **Heap-Local**: Collection occurs per-heap, not globally
- **Concurrent**: Collection can occur during execution

### Memory Safety

- **Bounds Checking**: All array accesses are validated
- **Type Safety**: Runtime type checking prevents corruption
- **Null Safety**: Explicit null checks before dereference
- **Stack Protection**: Stack overflow detection and handling

## Calling Convention

### Function Calls

1. **Arguments**: Pushed onto stack in left-to-right order
2. **Call Instruction**: Transfers control and pushes return address
3. **Local Setup**: Function allocates local variable slots
4. **Execution**: Function body executes with access to locals and stack
5. **Return**: Return instruction pops return address and transfers control

### Stack Frame Layout

```
| Local Variables |  (slots 0-N)
| Arguments       |  (passed on evaluation stack)
| Return Address  |  (on call stack)
| Previous Frame  |  (caller's context)
```

## Security Considerations

### Sandboxing

- **Heap Isolation**: Prevents cross-heap data access
- **Instruction Validation**: All bytecode is validated before execution
- **Resource Limits**: Configurable limits on memory, stack depth, execution time
- **Exception Containment**: Exceptions cannot escape their handling context

### Code Integrity

- **Magic Number**: Validates file format authenticity
- **Version Checking**: Ensures compatibility between compiler and runtime
- **Constant Validation**: All constant pool references are bounds-checked
- **Address Validation**: All jump targets are validated as valid instruction addresses

## Performance Characteristics

### Optimization Features

- **Stack-Based**: Efficient for expression evaluation
- **Compact Encoding**: Minimal bytecode size
- **Fast Dispatch**: Direct opcode-to-handler mapping
- **Local Caching**: Local variables provide fast access patterns

### Scalability

- **Multi-Heap**: Enables parallel execution with isolation
- **Incremental GC**: Reduces pause times for large heaps
- **Streaming**: Bytecode can be parsed incrementally
- **Modular**: Code organization supports efficient loading

## Version History

### Version 1.0.0 (Current)
- Initial release
- Complete instruction set implementation
- Structured exception handling
- Multi-heap architecture
- Stack-based execution model

## Implementation Notes

### Bytecode Generation

Compilers targeting the loaf VM should:
- Generate valid bytecode headers with correct magic numbers
- Ensure all constant pool references are valid
- Implement proper exception handling block nesting
- Validate instruction operand counts and types

### Runtime Implementation

VM implementations should:
- Validate all bytecode before execution
- Implement proper stack overflow protection
- Support all required data types and operations
- Provide configurable resource limits
- Implement efficient garbage collection

### Debugging Support

The bytecode format supports debugging through:
- Instruction-level stepping
- Stack inspection
- Local variable examination
- Exception stack trace generation
- Heap state visualization

---

This specification defines the complete loaf bytecode format and virtual machine architecture. For implementation details and examples, refer to the reference implementation in the loaf runtime.
