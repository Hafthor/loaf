# Copilot Instructions for loaf Language Compiler

<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

## Project Overview
This is a Rust-based compiler for the loaf programming language - a declarative JSON-style language that generates bytecode compatible with the loaf runtime.

## Language Features
- **Declarative JSON-style syntax**: Similar to XSLT but using JSON instead of XML
- **Forward immutable assignments**: Variables can reference other variables defined later in the code
- **Implicit typing**: Types are inferred automatically
- **Promise types**: Automatic deferred resolution where dependencies become promises
- **HTTP endpoints**: Easy method-to-endpoint conversion with isolated heaps
- **Heap isolation**: Each HTTP endpoint gets its own heap that's destroyed after response

## Architecture
- **Lexer**: Tokenizes JSON-style source code
- **Parser**: Builds AST from tokens
- **Semantic Analyzer**: Resolves forward references, infers types, handles promises
- **Code Generator**: Emits bytecode compatible with the loaf runtime
- **HTTP Runtime**: Manages endpoint registration and heap isolation

## Code Style Guidelines
- Use Rust best practices with proper error handling
- Implement comprehensive tests for each component
- Use clear, descriptive names for types and functions
- Document all public APIs with examples
- Handle all error cases gracefully with proper error types

## Dependencies
- The compiler should generate bytecode compatible with the existing loaf runtime
- Use standard Rust libraries where possible
- Consider serde for JSON handling
- Use tokio for async HTTP capabilities
