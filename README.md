# Gila

```
Vec type
    x: u32
    y: u32
end


vecs = [Vec(1,2), Vec(3,4)]

print(vecs)
```

## Todo

### Non functional features

- field constructors are wrong
- loading prelude has all sorts of weird behaviour
- instruction data is all in the enum
- macros for fetching instructions and counters etc
- parsing can 'consume' tokens and error
- string constant duplication fixed
- bytecode caching

### Language Features

- booleans
- generics
- builtin result type
- lhs struct field assignment
- testing
- asserts
- matches
- iterators
- intrinsics without the special syntax
- recursion
- match
- nice slicing operations i.e. equality checks
- strict typing
- shadowing variables
- closure capture setting
- varying integer sizes
- floating point support
- generics
- std lib
- prelude
- function args
  - default args
- multiple return values
- blocks are their own thing i.e. `do end` because right now theyre built in to
  if's
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
