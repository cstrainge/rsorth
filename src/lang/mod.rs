
/// Module for managing the original source code.
pub mod source_buffer;

/// Module for managing the turning of the source code into a list of tokens for further processing.
pub mod tokenizing;

/// Module for defining the byte-code instruction at the operations of the Strange Forth virtual
/// machine.
pub mod code;

/// Module for compiling that list of tokens into a list of byte-code instructions for the
/// interpreter to execute.  Due to the nature of the Strange Forth language some words will be
/// executed as others are being compiled.  This is why this phase requires an active interpreter
/// in order to compile the code.
///
/// That is, the code being compiled may help in the compiling of the code.
pub mod compilation;
