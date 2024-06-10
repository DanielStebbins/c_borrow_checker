# c_borrow_checker
A partial borrow-checker (a form of static analysis) for C programs. Based on Rust and written in Rust.

This project is not meant to be a full borrow-checker, but rather a proof of concept for central C language features such as structs, pointers, if/else statements, function calls, and loops.

The four Rust source code files can be found in /src.
  - main.rs is the main file, where the path to the C file to be analyzed can be editied.
  - The name of the function(s) to analyze must be included in the initialization of the BorrowChecker.
  - The PrintType enums can also be changed to give different outputs.

All test inputs can be found in /inputs.
  - /inputs/development has over 20 small tests based on the Rust compiler output that I used to guide the development process.
  - /inputs/kernel0 and /inputs/kernel1 contain a mixture of C and Rust files showing different versions of the same Linux kernel functions.

An example AST output can be found in ast.txt.

See the full project report [here](Final Project Report.pdf).
