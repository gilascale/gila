# Gila


## Todo

### Non functional features
- instruction data is all in the enum
- macros for fetching instructions and counters etc
- parsing can 'consume' tokens and error
- string constant duplication fixed
- bytecode caching

### Language Features
- generics
- std lib
- prelude
- function args
    - default args
- multiple return values
- blocks are their own thing i.e. `do end` because right now theyre built in to if's
- module system
- loops
- lists/slices
- JIT
- error handling
    - stack traces
    - print error locations
- sandboxing

## Bugs
- Can't seem to have multiple symbols in repl