
/// Mostly words that are used to change or read the state of the interpreter.
mod sorth_words;

/// Words that manipulate the data stack.
mod stack_words;

/// Simple constants.
mod constant_words;

/// Words that manipulate and generate byte-code.
mod bytecode_words;

/// Words that work with words.
mod word_words;

/// Words that create new words.
mod word_creation_words;

/// Words that work with Value types.
mod value_type_words;

/// Words that work with strings.
mod string_words;

/// Words that work with data structures.
mod data_structure_words;

/// Words that work with arrays.
mod array_words;

/// Words that work with byte buffers.
mod byte_buffer_words;

/// Words that work with hash tables.
mod hash_table_words;

/// Words that work with math, logic, bit manipulation and Value equality.
mod math_logic_and_bit_words;



use crate::runtime::{ built_ins::base_words::{
                                      sorth_words::register_sorth_words,
                                      stack_words::register_stack_words,
                                      constant_words::register_constant_words,
                                      bytecode_words::register_bytecode_words,
                                      word_words::register_word_words,
                                      word_creation_words::register_word_creation_words,
                                      value_type_words::register_value_type_words,
                                      string_words::register_string_words,
                                      data_structure_words::register_data_structure_words,
                                      array_words::register_array_words,
                                      byte_buffer_words::register_byte_buffer_words,
                                      hash_table_words::register_hash_table_words,
                                      math_logic_and_bit_words::register_math_logic_and_bit_words },
                      interpreter::Interpreter};


/// Called to register all of the core words of the language.
pub fn register_base_words(interpreter: &mut dyn Interpreter)
{
    register_sorth_words(interpreter);
    register_stack_words(interpreter);
    register_constant_words(interpreter);
    register_bytecode_words(interpreter);
    register_word_words(interpreter);
    register_word_creation_words(interpreter);
    register_value_type_words(interpreter);
    register_string_words(interpreter);
    register_data_structure_words(interpreter);
    register_array_words(interpreter);
    register_byte_buffer_words(interpreter);
    register_hash_table_words(interpreter);
    register_math_logic_and_bit_words(interpreter);
}
