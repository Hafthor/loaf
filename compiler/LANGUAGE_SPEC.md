# loaf Language Specification

**Version 0.1.0**  
**Date: May 25, 2025**

## Overview

loaf is a declarative, JSON-style programming language designed for creating reactive applications with automatic dependency resolution, promise-based asynchronous operations, and HTTP endpoint generation. The language emphasizes simplicity, readability, and automatic type inference.

## Design Principles

- **Declarative Syntax**: Similar to JSON but with enhanced expressiveness
- **Forward References**: Variables can reference other variables defined later in the code
- **Implicit Typing**: Types are inferred automatically from values and expressions
- **Promise-based Async**: Automatic handling of asynchronous operations with promises
- **HTTP Endpoints**: Easy conversion from methods to REST endpoints
- **Heap Isolation**: Each HTTP endpoint operates in its own isolated memory space

## Language Syntax

### Basic Structure

loaf programs are structured as JSON-like objects with unquoted keys and enhanced expression support:

```loaf
{
  key1: value1,
  key2: expression,
  key3: "string literal"
}
```

### Data Types

#### Primitive Types

```loaf
{
  // Numbers (integers and floats)
  age: 25,
  price: 19.99,
  negative: -10,
  
  // Strings (must be quoted)
  name: "Alice",
  message: "Hello, World!",
  
  // Booleans
  isActive: true,
  isComplete: false,
  
  // Null
  empty: null
}
```

#### Complex Types

```loaf
{
  // Objects
  user: {
    name: "Bob",
    age: 30,
    active: true
  },
  
  // Arrays
  numbers: [1, 2, 3, 4, 5],
  names: ["Alice", "Bob", "Charlie"],
  mixed: [1, "hello", true, null]
}
```

### Expressions

#### Arithmetic Operations

```loaf
{
  sum: 10 + 5,
  difference: 20 - 8,
  product: 4 * 7,
  quotient: 15 / 3,
  
  // Complex expressions
  result: (10 + 5) * 2 - 3
}
```

#### String Operations

```loaf
{
  greeting: "Hello, " + name,
  fullName: firstName + " " + lastName,
  message: "User " + userId + " is active"
}
```

#### Member Access

```loaf
{
  user: {
    profile: {
      name: "Alice",
      email: "alice@example.com"
    }
  },
  
  // Access nested properties
  userName: user.profile.name,
  userEmail: user.profile.email,
  
  // Chain member access
  displayName: user.profile.name + " (" + user.profile.email + ")"
}
```

### Forward References

loaf supports forward references, allowing variables to reference other variables defined later:

```loaf
{
  // greeting references userName which is defined later
  greeting: "Hello, " + userName,
  
  // userName references user.name
  userName: user.name,
  
  // user is defined last but can be referenced above
  user: {
    name: "Alice",
    age: 30
  }
}
```

### Annotations

Annotations are special directives that provide metadata and behavior to the loaf runtime.

#### Promise Annotations

Promise annotations enable asynchronous operations with automatic dependency resolution:

```loaf
{
  userId: 123,
  
  // Promise that will resolve asynchronously
  userData: fetch_user(userId),
  
  // This will automatically become a promise since it depends on userData
  userName: userData.name,
  
  // String concatenation with promise values
  greeting: "Welcome, " + userName
}
```

#### Endpoint Annotations

Endpoint annotations convert variables into HTTP endpoints:

```loaf
{
  // Define the response data
  apiResponse: {
    message: "Hello from loaf!",
    timestamp: 1640995200,
    version: "1.0.0"
  },
  
  // Create an HTTP GET endpoint
  helloEndpoint: @endpoint:GET:/api/hello
}
```

#### HTTP Call Annotations

HTTP call annotations enable external API calls:

```loaf
{
  // Make an HTTP GET request
  currentTime: @http:GET:https://api.time.com/now,
  
  // HTTP POST with body
  userCreation: @http:POST:https://api.example.com/users,
  
  // The endpoint will automatically use appropriate request data
  apiCall: @http:PUT:https://api.example.com/data
}
```

## Type System

### Automatic Type Inference

loaf automatically infers types from values and expressions:

```loaf
{
  // Inferred as number
  count: 42,
  
  // Inferred as string
  message: "Hello",
  
  // Inferred as boolean
  isReady: true,
  
  // Inferred as object with typed fields
  user: {
    name: "Alice",    // string
    age: 30,          // number
    active: true      // boolean
  },
  
  // Inferred from expression
  total: price * quantity,  // number if both operands are numbers
  
  // Member access inherits type from source
  userName: user.name       // string (from user.name)
}
```

### Promise Types

When a variable depends on a promise, it automatically becomes a promise type:

```loaf
{
  // userData is Promise<Object>
  userData: fetch_user(123),
  
  // userName becomes Promise<String> because userData is a promise
  userName: userData.name,
  
  // greeting becomes Promise<String> because it depends on userName
  greeting: "Hello, " + userName
}
```

### Type Propagation

Types propagate through expressions and references:

```loaf
{
  user: {
    profile: {
      name: "Alice",     // string
      age: 30            // number
    }
  },
  
  // Inherits string type from user.profile.name
  displayName: user.profile.name,
  
  // Results in string type from concatenation
  fullGreeting: "Welcome, " + displayName,
  
  // Results in number type from arithmetic
  nextYear: user.profile.age + 1
}
```

## Dependency Resolution

### Automatic Dependency Analysis

loaf automatically analyzes dependencies between variables and resolves them in the correct order:

```loaf
{
  // Dependencies: fullName -> firstName + lastName
  fullName: firstName + " " + lastName,
  
  // Dependencies: greeting -> fullName -> firstName + lastName  
  greeting: "Hello, " + fullName,
  
  // Base dependencies (no dependencies)
  firstName: "Alice",
  lastName: "Smith"
}
```

**Resolution Order:**
1. `firstName` (no dependencies)
2. `lastName` (no dependencies)
3. `fullName` (depends on firstName, lastName)
4. `greeting` (depends on fullName)

### Circular Dependency Detection

loaf detects and reports circular dependencies:

```loaf
// This would cause a circular dependency error
{
  a: b + 1,
  b: c + 1,
  c: a + 1  // Error: circular dependency between a, b, c
}
```

### Promise Dependency Handling

Promises are resolved asynchronously but maintain dependency order:

```loaf
{
  userId: 123,
  
  // Step 1: Initiate promise
  userData: fetch_user(userId),
  
  // Step 2: Wait for userData, then access property (becomes promise)
  userName: userData.name,
  
  // Step 3: Wait for userName, then concatenate (becomes promise)
  greeting: "Hello, " + userName
}
```

## HTTP Endpoints

### Endpoint Declaration

Endpoints are declared using the `@endpoint` annotation:

```loaf
{
  // Define response data
  welcomeMessage: {
    message: "Welcome to our API",
    timestamp: @http:GET:https://api.time.com/now,
    version: "1.0.0"
  },
  
  // Create GET endpoint at /api/welcome
  welcomeEndpoint: @endpoint:GET:/api/welcome,
  
  // Create POST endpoint at /api/data
  dataEndpoint: @endpoint:POST:/api/data,
  
  // Create PUT endpoint with path parameters
  updateEndpoint: @endpoint:PUT:/api/users/:id
}
```

### Endpoint Isolation

Each endpoint runs in its own isolated heap:

- Memory is allocated per request
- Variables are scoped to the request lifecycle
- Heap is destroyed after response is sent
- No shared state between requests

### HTTP Methods

Supported HTTP methods:
- `GET` - Retrieve data
- `POST` - Create new resources
- `PUT` - Update existing resources
- `DELETE` - Remove resources
- `PATCH` - Partial updates

## Runtime Behavior

### Execution Model

1. **Parse Phase**: Source code is tokenized and parsed into an AST
2. **Analysis Phase**: Dependencies are resolved and types are inferred
3. **Compilation Phase**: Bytecode is generated for the loaf runtime
4. **Execution Phase**: Bytecode is executed with promise resolution

### Memory Management

- **Heap Isolation**: Each HTTP endpoint gets its own heap
- **Automatic Cleanup**: Memory is automatically freed after request completion
- **Promise Caching**: Promise results are cached within the request scope

### Error Handling

loaf provides comprehensive error reporting:

```
Error Types:
- Syntax Errors: Invalid language syntax
- Parse Errors: Malformed expressions or structures
- Type Errors: Type mismatches or invalid operations
- Dependency Errors: Circular dependencies or unresolved references
- Runtime Errors: Promise failures or HTTP errors
```

## Examples

### Simple Data Processing

```loaf
{
  // Input data
  rawData: [1, 2, 3, 4, 5],
  multiplier: 2,
  
  // Processing (would need array operations in full implementation)
  total: 15,  // sum of rawData
  average: total / 5,
  scaled: average * multiplier,
  
  // Output
  result: {
    original: rawData,
    processed: scaled,
    metadata: {
      count: 5,
      average: average
    }
  }
}
```

### API with Promises

```loaf
{
  // Request parameters
  userId: 123,
  includeProfile: true,
  
  // Async data fetching
  userData: @promise:fetch_user(userId),
  userProfile: @promise:fetch_profile(userId),
  
  // Data composition
  response: {
    user: userData,
    profile: userProfile,
    fullName: userData.firstName + " " + userData.lastName,
    isActive: userData.status == "active"
  },
  
  // HTTP endpoint
  apiHandler: @endpoint:GET:/api/users/:userId
}
```

### Microservice Endpoint

```loaf
{
  // Service configuration
  serviceInfo: {
    name: "User Service",
    version: "1.2.0",
    uptime: @http:GET:https://internal.api.com/uptime
  },
  
  // Health check response
  healthResponse: {
    status: "healthy",
    service: serviceInfo.name,
    version: serviceInfo.version,
    timestamp: @http:GET:https://api.time.com/now,
    uptime: serviceInfo.uptime
  },
  
  // Endpoints
  healthEndpoint: @endpoint:GET:/health,
  statusEndpoint: @endpoint:GET:/status
}
```

## Best Practices

### Code Organization

1. **Group Related Data**: Keep related variables together
2. **Use Descriptive Names**: Choose clear, meaningful variable names
3. **Minimize Dependencies**: Reduce complex dependency chains when possible
4. **Comment Complex Logic**: Use comments for complex expressions

### Performance Considerations

1. **Promise Batching**: Related promises are automatically batched
2. **Lazy Evaluation**: Values are computed only when needed
3. **Memory Efficiency**: Use heap isolation for better memory management
4. **Caching**: Promise results are cached within request scope

### Error Prevention

1. **Avoid Circular Dependencies**: Design data flow to be acyclic
2. **Type Consistency**: Ensure operations match expected types
3. **Null Safety**: Handle potential null values appropriately
4. **Promise Error Handling**: Consider promise failure scenarios

## Tooling

### Compiler Commands

```bash
# Compile loaf source to bytecode
loaf compile --input program.loaf --output program.crouton

# Run loaf program directly
loaf run --input program.loaf

# Start HTTP server with endpoints
loaf server --input endpoints.loaf --port 4271

# Show program information
loaf info --input program.loaf --symbols --deps
```

### Development Workflow

1. Write loaf source code (`.loaf` files)
2. Compile to bytecode (`.crouton` files) 
3. Run with loaf runtime
4. Deploy as HTTP service

## Future Extensions

### Planned Features

- Array operations and transformations
- Pattern matching and destructuring
- Module system and imports
- Custom function definitions
- Advanced HTTP middleware
- WebSocket support
- Database integrations

### Language Evolution

The loaf language is designed to evolve incrementally while maintaining backward compatibility. Future versions will expand functionality while preserving the core declarative philosophy and JSON-like syntax.

---

*This specification covers loaf Language version 0.1.0. For the latest updates and examples, visit the official loaf documentation.*
