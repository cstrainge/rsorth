
/// Module contains the Value enumeration and it's implementation.  The value is one of the core
/// data structures of the interpreter.  It is used to represent all data types that the interpreter
/// and underlying Forth code can understand and manage.
pub mod value;


/// Hold the ContextualData trait, used for managing contexts in the interpreter.
pub mod contextual_data;

/// A list that can be used in a contextual manner.  This is useful for the interpreter to keep
/// track of things like variables that can be allocated and released as part of script word
/// contexts.
pub mod contextual_list;

/// The dictionary module provides the core interpreter word dictionary used by the Strange Forth
/// interpreter
pub mod dictionary;


/// Module that holds the DataObject and DataObjectDefinition data structures.  These are used to
/// represent structured data within Strange Forth scripts.
pub mod data_object;

/// Represent a vector of values useable by scripts in the runtime.
pub mod value_vec;

/// The hash table type used by Forth scripts for storing related data.
pub mod value_hash;


/// Module for the ByteBuffer data structure.
pub mod byte_buffer;
