
mod sorth_words;
mod stack_words;
mod constant_words;
mod bytecode_words;
mod word_words;
mod word_creation_words;
mod value_type_words;
mod string_words;
mod data_structure_words;
mod array_words;
mod byte_buffer_words;
mod hash_table_words;
mod math_logic_and_bit_words;
mod equality_words;



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
                                        math_logic_and_bit_words::register_math_logic_and_bit_words,
                                        equality_words::register_equality_words },
                      interpreter::Interpreter};



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
    register_equality_words(interpreter);
}
