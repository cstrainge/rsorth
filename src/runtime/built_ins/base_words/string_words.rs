
use std::sync::atomic::{ AtomicUsize, Ordering };
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
                        error::{ self,  script_error },
                        interpreter::Interpreter } };



fn byte_to_char_index(interpreter: &mut dyn Interpreter,
                      string: &str,
                      byte_index: usize) -> error::Result<usize>
{
    if !string.is_char_boundary(byte_index)
    {
        script_error(interpreter, format!("Byte index {} is not a valid character boundary.",
                                          byte_index))?
    }

    let character_index = string[..byte_index].chars().count();
    Ok(character_index)
}

fn char_index_to_byte_index(interpreter: &mut dyn Interpreter,
                            string: &str,
                            char_index: usize) -> error::Result<usize>
{
    let total_chars = string.chars().count();

    if char_index > total_chars
    {
        script_error(interpreter, format!("Character index {} is out of range for string {}.",
                                          char_index,
                                          string))?
    }

    let byte_index = string.char_indices()
                           .nth(char_index)
                           .map(|( i, _ )| i)
                           .unwrap_or(string.len());

    Ok(byte_index)
}



fn word_string_length(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let string = interpreter.pop_as_string()?;
    let length = string.chars().count() as i64;

    interpreter.push(length.to_value());
    Ok(())
}

fn word_string_insert(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let mut string = interpreter.pop_as_string()?;
    let mut index = interpreter.pop_as_int()?;
    let sub_string = interpreter.pop_as_string()?;

    if    index > string.chars().count() as i64
       || index < 0
    {
        index = string.chars().count() as i64 - 1;
    }

    string.insert_str(index as usize, &sub_string[0..sub_string.len()]);
    interpreter.push(string.to_value());

    Ok(())
}

fn word_string_remove(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let mut string = interpreter.pop_as_string()?;
    let position = interpreter.pop_as_int()?;
    let mut count = interpreter.pop_as_int()?;

    let char_indices: Vec<usize> = string.char_indices().map(|(i, _)| i).collect();
    let char_count = char_indices.len();

    if    position >= char_count as i64
       || position < 0
    {
        script_error(interpreter, format!("Position {} is out of range for string of length {}.",
                                          position,
                                          char_count))?;
    }

    if    count < 0
       || ((position + count) >= char_count as i64)
    {
        count = char_count as i64 - position - 1;
    }

    let start_byte = char_indices[position as usize];
    let end_byte = char_indices[(position + count) as usize];

    string.drain(start_byte..=end_byte);

    interpreter.push(string.to_value());

    Ok(())
}

fn word_string_find(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let string = interpreter.pop_as_string()?;
    let search_string = interpreter.pop_as_string()?;

    let byte_index = string.find(&search_string);

    if let Some(byte_index) = byte_index
    {
        let char_index = byte_to_char_index(interpreter, &string, byte_index)?;
        interpreter.push(char_index.to_value());
    }
    else
    {
        interpreter.push((-1 as i64).to_value());
    }


    Ok(())
}

fn word_string_index_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let string = interpreter.pop_as_string()?;
    let char_index = interpreter.pop_as_int()?;
    let char_count = string.chars().count();

    if    char_index < 0
       || char_index as usize >= char_count
    {
        script_error(interpreter, format!("Character index {} is out of range for string {}.",
                                          char_index,
                                          char_count))?;
    }

    let byte_index = char_index_to_byte_index(interpreter, &string, char_index as usize)?;
    let character = string[byte_index..].chars().next().unwrap();

    interpreter.push(character.to_string().to_value());

    Ok(())
}

fn word_string_to_number(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let string = interpreter.pop_as_string()?;

    if let Some(_) = string.find(".")
    {
        let number = string.parse::<f64>();

        match number
        {
            Ok(value)  => interpreter.push(value.to_value()),
            Err(error) => script_error(interpreter,
                                       format!("Could not convert string {} to number: {}.",
                                               string,
                                               error))?
        }
    }
    else
    {
        let number = string.parse::<i64>();

        match number
        {
            Ok(value)  => interpreter.push(value.to_value()),
            Err(error) => script_error(interpreter,
                                       format!("Could not convert string {} to number: {}.",
                                               string,
                                               error))?
        }
    }

    Ok(())
}

fn word_to_string(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let string = interpreter.pop()?.to_string();

    interpreter.push(string.to_value());
    Ok(())
}

fn word_hex(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    let number =
        if value.is_float()
        {
            let f_value = value.as_float(interpreter)?;
            f_value.to_bits() as i64
        }
        else if value.is_numeric()
        {
            value.get_int_val()
        }
        else if value.is_string()
        {
            let value = value.get_string_val();

            if value.len() == 1
            {
                value.chars().next().unwrap() as i64
            }
            else
            {
                0
            }
        }
        else
        {
            return script_error(interpreter, format!("Value {} is not a number.", value));
        };

    interpreter.push(format!("{:x}", number).to_value());
    Ok(())
}

fn word_unique_str(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    let index = INDEX.fetch_add(1, Ordering::SeqCst);
    let unique_str = format!("unique-str-{:08x}", index);

    interpreter.push(unique_str.to_value());
    Ok(())
}



pub fn register_string_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "string.size@", word_string_length,
        "Get the length of a given string.",
        "string -- size");

    add_native_word!(interpreter, "string.[]!", word_string_insert,
        "Insert a string into another string.",
        "string -- updated_string");

    add_native_word!(interpreter, "string.remove", word_string_remove,
        "Remove some characters from a string.",
        "string -- updated_string");

    add_native_word!(interpreter, "string.find", word_string_find,
        "Find the first instance of a string within another. Index if found, npos if not.",
        "search_string string -- result");

    add_native_word!(interpreter, "string.[]@", word_string_index_read,
        "Read a character from the given string.",
        "index string -- character");

    add_native_word!(interpreter, "string.to_number", word_string_to_number,
        "Convert a string into a number.",
        "string -- number");

    add_native_word!(interpreter, "to_string", word_to_string,
        "Convert a value to a string.",
        "value -- string");

    add_native_word!(interpreter, "hex", word_hex,
        "Convert a number into a hex string.",
        "number -- hex_string");

    add_native_word!(interpreter, "unique_str", word_unique_str,
        "Generate a unique string and push it onto the data stack.",
        " -- string");

    add_native_word!(interpreter, "string.npos",
        |interpreter| { interpreter.push((-1 as i64).to_value()); Ok(()) },
         "Constant value that indicates a search has failed.",
         " -- npos");
 }
