
/// All of the core data structures used by the Strange Forth interpreter.
pub mod data_structures;

/// Module for defining the built-in native words that are available to the Strange Forth
/// interpreter.
pub mod built_ins;

/// Module for defining the error reporting of the Strange Forth interpreter.
pub mod error;

/// Module for defining the core functionality of the Strange Forth interpreter.  This includes
/// tools for managing and examining the interpreter's state.
pub mod interpreter;
