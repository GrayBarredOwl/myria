# Myria
Myria is a small interpreted programming language. It is my second attempt at making a language, and first that advanced past variable creation. Myria's syntax is primarily based on Swift, Python, and my own ideas. Myria's name comes from the obsolete metric unit prefix for 10,000

## Usage
To run the interpreter, do `cargo run {script-path}` to exexute a file, or `cargo run` to enter the REPL. Myria scripts typically end in the _.myria_ extension. In the REPL, backslash enter can be used to spread the expression over multiple lines. If you want to load a dynamic library, use the --lib flag, with the path to the library(--lib myfolder/libmylib.dylib)

## Myria syntax
### Core syntax rules:
Myria is an expression based language. As such ifs, loops, and blocks can be treated like values. Blocks evaluate to the final statement, ifs to the final expression, and loops to the result of the last line that ran. New lines are significant, although semi colons can be used to the exact same effect. Line comments are made using #. All characters must be in ASCII. Execution begins at the start; no main function is needed. Blocks are enclosued in curly brackets.

### Core data types:
Mria has a few primitive types: Null, Int, Float, Char, Bool, List, Type, Function, Instance. Null is a singleton with 1 value, the _null_ keyword. Ints and Floats both use the 64 bit integers and floating point numbers. Char is a unicode codepoint. Bool is either _true_ or _false_. Lists are sequences of elements. These elements do not have to be the same type. Lists are declared inside brackets ([]) with elements comma-seperated. Strings are homogenous lists of Chars. Type represents any of the types above. It is used for casting. Functions are functions. Instances are objects, and can contain fields. An empty Instance can be made with the standard library `empty()` function, there is no instance literal syntax.

### Variables:
Myria has 2 types of variables: mutables, and constants. mutables are declared with the _var_ keyword, whilst constants are delcared with _let_.

Valid identifers: the first character is a-z, A-Z, _, and any subsequent characters may also include 0-9.

Variables are assigned using _=_. The compound assignment operators also work (+=, -=, ...). Variables must be declared exactly once. any variable (including constants) can be left uninitialized and given a value later.

### Operators:
Myria supports the core 4 arithmetic operations (+, -, *, /), and the six comparision operators (==, !=, &gt, <, >=, <=), and three boolean operations: (&&, ||, !). Note: && and || do not short-circuit. The comparision operators are implemented in terms of <, and ==; All others are combinations of them, as a result, an impure expression may be evaluated multiple times. 

### Core constructs
Conditionals.
Loops.

### Lists
Lists are specified with brackets. The contents of the list are placed inside the brackets, seperated by commas. You can have an optional trailing comma. list elements can be accessed using .index. `let m = "Hello, world"; 'H' == m.0`. Lists are 0-indexed. Strings are a type of list. The length can be obtained by casting the list to an Int.

### Casting
Casting is done by calling a Type like a function. Sample `50 == Int(50.55)`. Casting a list to an Int gets the length `2 == Int("94")`. This can be done to get the type of an expression: `Char == Type('c')`, `Type == Type(Type("type"))`.

### Modules
Import files with the `import(path)` function. It takes a string for the file you want to execute (without the extension), and executes it returning the final expression of the file. Most files intended to be imported end by returning an Instance containing all exported variables, and methods (similar to Lua). Dynamic can be loaded using the --lib flag [usage](#usage)

### Functions
Functions are created with the _func_ keyword, followed by optional comma-seperated parameters. Then the body which is a block. examples: `let five = func() { 5 }`, `let sum = func(x, y) { print("adding", x, "and", y); x + y }`. All functions are pass by copy. recursion can be done calling the _recurs_ keyword.

### Insatances & Classes
The _class_ keyword is just syntax sugar around a function that creates an empty instance named self, and returns it.  Instances. Fields can be accesed like so: _name.field_name_. There can not be any spaces between the variable and field name. New fields are declared like variables: `let inst.new_field = 20`. Fields, like variables can be mutable, immutable, initialized, or uninitialized. 

### Rust FFI
Dynamic library plugins are supported on MacOS. Pass the file path in the --lib flag. Many common declarations can be found in with `use myria::plugin::{...}`. The library must have the following:
* fn load() -> Vec&ltmyria::plugin::FuncInfo&gt
* fn name() -> String
The functions are can be accessed using the built-in _rsc_ function. It takes the function identifier string, and then all argumnets. The identifer string goes is the function name (in the FuncInfo object), prefixed by _name._, where name is from the name function. `rsc("plugin-name.function", arg1, arg2, ...)`, where name() -> "plugin-name", and the funcInfo contained the name "function". Loaded functions can be listed by calling `list_rsc()`.

### Standard Library

#### Built-ins
* print -> print arguments, any number of arguments
* dbg_print -> print arguments with debug formatting, any number of arguments
* file -> read specified file to string, 1 argument: path
* exit -> exit Myria, 0 or 1 argument: error code
* rsc -> Rust FFI, any number of arguments after the identifer string for the Rust function
* list_rsc" -> get list of all rsc identifer strings, no argumnets
* eval -> Evaluate string of Myria code (variables and libraries are not shared), 1 argument: Myria code string to be run
* import -> Executes myria code, returns the final expression result, 1 argument: path
* mod -> Modulus operation, 2 arguments: first and second numbers
* empty -> Create an empty Instance, no arguments
* str -> Converts the argument to a string, 1 argument
* get -> Gets an element from a list, 2 arguments: list, index (negative index like Python works)
* replace -> returns a new list with the value at the given index replaced: 3 arguments: list, index to replace, new value

### In the modules
#### myrialib/file

* READ, WRITE, APPEND -> File open modes
* SEEK_START, SEEK_CURRENT, SEEK_END -> seek offset modes

* File -> class representing an open file, 2 arguments, path, open mode(sum of READ, WRITE, and APPEND)
* close -> closes the file, 1 argument: File Instance
* read -> reads the file, 2 arguments: File Instance, Int amount
* write -> writes to the file, 2 arguments: File Instance, String to write
* seek -> set file position, 3 arguments: File Instance, Seek mode (one of SEEK_START, SEEK_CURRENT, or SEEK_END), Int offset amount