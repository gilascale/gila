# Gila

```
std =import std
print = std.io.print

Vec type
    x: u32
    y: u32
end

display fn(self:Vec) do
  print("im a vec "+self)
end

v = Vec(x=1,y=2)
v.display()
```

```
0    std =import std
                                                                 LOAD_CONST  0  0  0
                                                                 LOAD_CONST  6  0  6
                                                                     IMPORT  6  7  0
1    print = std.io.print
                                                                 LOAD_CONST  7  0  8
                                                              STRUCT_ACCESS  7  8  9
                                                                 LOAD_CONST  8  0 10
                                                              STRUCT_ACCESS  9 10 11
2    
3    Vec type
                                                                 LOAD_CONST  9  0 12
4        x: u32
                                                                 LOAD_CONST  1  0  1
5        y: u32
6    end
7    
8    display fn(self:Vec) do
                                                                 LOAD_CONST  2  0  2
                                                                 LOAD_CONST 10  0 13
                                                                   BUILD_FN 13  0  0
9      print("im a vec "+self)
10   end
11   
12   v = Vec(x=1,y=2)
                                                                       ADDI  0  1 14
                                                                       ADDI  0  2 15
                                                                 LOAD_CONST 11  0 18
                                                                    CALL_KW 12 18 14
13   v.display()                                                                 LOAD_CONST  3  0  3
                                                                   BUILD_FN  3  0  0
                                                                 LOAD_CONST 12  0 19
                                                              STRUCT_ACCESS 16 19 20
                                                                       CALL 20 21  0
```

## Todo

### Bugs

- fix tuple parsing...
- methods with self and an arg don't work due to calling convention
  - i suspect its because were not allocating space for the extra self at
    compile time
- we are loading DLLs twice
- cant call returned functions i.e. some_fn()()
- the prelude is being added into the dumped bytecode file as its the same line
  as some of the other code
- need a return statement otherwise subsequent calls dont work
- structs with no constructors don't work and loop/hang

### Non functional features

- restructure the contexts, i think we need to start cloning them and returning
  them as we can't just be passing references and stuff around.
- reuse registers
- add implicit returns to functions without them
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

- implicit returns
- void types
- tuple unpacking
- tests have multiple asserts in
- test names can only be one word
- capturing in closures
- easy way to add builtin modules/files (i.e. socket library)
- groups i.e. 3 + (4+3)
- garbage collection
- pattern matching
- tuple unpacking
- breaks
- class methods
- including other std stuff in std (circular import caching)
- std type hints
  - printable interface for print
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
- lex/parse/compile atoms
- import supports non-required assignment
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
