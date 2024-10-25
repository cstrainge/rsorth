
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue, error, interpreter::Interpreter } };



fn word_value_is_none(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_none().to_value());

    Ok(())
}

fn word_value_is_number(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_numeric().to_value());

    Ok(())
}

fn word_value_is_boolean(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_bool().to_value());

    Ok(())
}

fn word_value_is_string(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_string().to_value());

    Ok(())
}

fn word_value_is_structure(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let _value = interpreter.pop()?;

    interpreter.push(false.to_value());

    Ok(())
}

fn word_value_is_array(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_vec().to_value());

    Ok(())
}

fn word_value_is_buffer(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let _value = interpreter.pop()?;

    interpreter.push(false.to_value());

    Ok(())
}

fn word_value_is_hash_table(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_hash_map().to_value());

    Ok(())
}

fn word_value_is_token(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_token().to_value());

    Ok(())
}

fn word_value_is_code(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.is_code().to_value());

    Ok(())
}



pub fn register_value_type_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "value.is-none?", word_value_is_none,
        "Is the value nothing?",
        "value -- bool");

    add_native_word!(interpreter, "value.is-number?", word_value_is_number,
            "Is the value a number?",
            "value -- bool");

    add_native_word!(interpreter, "value.is-boolean?", word_value_is_boolean,
        "Is the value a boolean?",
        "value -- bool");

    add_native_word!(interpreter, "value.is-string?", word_value_is_string,
        "Is the value a string?",
        "value -- bool");

    add_native_word!(interpreter, "value.is-structure?", word_value_is_structure,
        "Is the value a structure?",
        "value -- bool");

    add_native_word!(interpreter, "value.is-array?", word_value_is_array,
        "Is the value an array?",
        "value -- bool");

    add_native_word!(interpreter, "value.is-buffer?", word_value_is_buffer,
        "Is the value a byte buffer?",
        "value -- bool");

    add_native_word!(interpreter, "value.is-hash-table?", word_value_is_hash_table,
        "Is the value a hash table?",
        "value -- bool");

    add_native_word!(interpreter, "value.is-token?", word_value_is_token,
            "Is the value a lexical token?",
            "value -- bool");

    add_native_word!(interpreter, "value.is-code?", word_value_is_code,
            "Is the value a block of bytecode?",
            "value -- bool");
}
