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

### Bugs

- struct constructor is still random
- print function calls just aren't executing

### Non functional features

- fix nested GCRefs in constant data
  - the issue is we do `init_constants()` which assigns a heap allocation to
    each GCRef in the constant pool. the problem is it doesn't then allocate
    stuff inside those constants.
  - maybe this isn't actually an issue... and we can just keep the ref to the
    constants? nah that wont work.
- constant hashmap so we don't keep on generating new constants (i.e. for bools,
  nums and strings)
- field constructors are wrong
- loading prelude has all sorts of weird behaviour
- instruction data is all in the enum
- macros for fetching instructions and counters etc
- parsing can 'consume' tokens and error
- string constant duplication fixed
- bytecode caching

### Language Features

- class methods
- including other std stuff in std (circular import caching)
- std type hints
  - printable interface for print
- named args
- tuples
- dictionaries
- try
- interface/prototype system
- enums
  - algebraic data types such as `Result = type $T | Error end`
    - this would require support for 'zero field' objects i.e. the field is
      implicit?
- virtual functions?
- target backend
- type-hint modules
- multiple params doesn't work
- lex/parse/compile atoms
- function return types
- import supports . in it
- import supports non-required assignment
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
