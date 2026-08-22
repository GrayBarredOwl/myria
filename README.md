# Myria
Myria is a small interpreted programming language. It is my second attempt at making a language, and first that advanced past variable creation. Myria's syntax is primarily based on Swift, Python, and my own ideas. Myria's name comes from the obsolete metric unit prefix for 10,000

## Usage
To run the interpreter, do `cargo run {script-path}` to exexute a file, or `cargo run` to enter the REPL. Myria scripts typically end in the _.myria_ extension. In the REPL, backslash enter can be used to spread the expression over multiple lines.

## Myria syntax
### Core syntax rules:
Myria is an expression based language. As such ifs, loops, and blocks can be treated like values. Blocks evaluate to the final statement, ifs to the final expression, and loops to the result of the last line that ran. New lines are significant, although semi colons can be used to the exact same effect. Line comments are made using #. All characters must be in ASCII. Execution begins at the start; no main function is needed. Blocks are enclosued in curly brackets.

### Core data types:
Mria has a few primitive types: Null, Int, Float, Char, Bool, List, Type, Function, Instance. Null is a singleton with 1 value, the _null_ keyword. Ints and Floats both use the 64 bit integers and floating point numbers. Char is a unicode codepoint. Bool is either _true_ or _false_. Lists are sequences of elements. These elements do not have to be the same type. Lists are declared inside brackets ([]) with elements comma-seperated. Strings are homogenous lists of Chars. Type represents any of the types above. It is used for casting. Functions are functions. Instances are instances of a class. An empty Instance can be made with the standard library `empty()` function.

### Variables:
Myria has 2 types of variables: mutables, and constants. mutables are declared with the _var_ keyword, whilst constants are delcared with _let_.

Valid identifers: the first character is a-z, A-Z, _, and any subsequent characters may also include 0-9.

Variables are assigned using _=_. The compound assignment operators also work (+=, -=, ...). Variables must be declared exactly once. any variable (including constants) can be left uninitialized and given a value later.

### Operators:
Myria supports the core 4 arithmetic operations (+, -, *, /), and the six comparision operators (==, !=, >, <, >=, <=), and three boolean operations: (&&, ||, !). Note: && and || do not short-circuit. The comparision operators are implemented in terms of <, and ==; All others are combinations of them, as a result, an impure expression may be evaluated multiple times. 

### Core constructs
Conditionals.
Loops.

### Lists
Lists are specified with brackets. The contents of the list are placed inside the brackets, seperated by commas. You can have an optional trailing comma. list elements can be accessed using .index. `let m = "Hello, world"; 'H' == m.0`. Lists are 0-indexed. 

### Casting
Casting is done by calling a Type like a function. Sample `50 == Int(50.55)`. Casting a list to an Int gets the length `2 == Int("94")`. This can be done to get the type of an expression: `Char == Type('c')`, `Type == Type(Type("type"))`.

### Modules
Import files with the `import(path)` function. It takes a string for the file you want to execute (without the extension), and executes it returning the final expression of the file. Most files intended to be imported end by returning an Instance containing all exported variables, and methods.
### Functions
Functions are created with the _func_ keyword, followed by optional comma-seperated parameters. Then the body which is a block. examples: `let five = func() { 5 }`, `let sum = func(x, y) { print("adding", x, "and", y); x + y }`

### Standard Library