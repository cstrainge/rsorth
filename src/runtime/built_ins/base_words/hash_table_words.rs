
use crate::{ add_native_word,
             location_here,
             runtime::{ data_structures::{ value::ToValue,
                                           value_hash::ValueHash },
                                           error::{ self,
                                                    script_error,
                                                    script_error_str },
                        interpreter::Interpreter } };



/// Create a new empty hash table.
///
/// Signature: ` -- hash-table`
fn word_hash_table_new(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let hash_table = ValueHash::new();

    interpreter.push(hash_table.to_value());
    Ok(())
}

/// Insert a value into a hash table.
fn word_hash_table_insert(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let hash_table = interpreter.pop_as_hash_map()?;
    let key = interpreter.pop()?;
    let value = interpreter.pop()?;

    if key.is_float()
    {
        script_error_str(interpreter, "Hash table keys can not be floating point numbers.")?;
    }

    hash_table.borrow_mut().insert(key, value);

    Ok(())
}

/// Extract a value from a hash table, error out if it isn't found.
///
/// Signature: `key table -- value`
fn word_hash_table_find(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let hash_table = interpreter.pop_as_hash_map()?;
    let key = interpreter.pop()?;

    if let Some(value) = hash_table.borrow().get(&key)
    {
        interpreter.push(value.clone());
    }
    else
    {
        script_error(interpreter, format!("Key {} not found in hash table.", key))?;
    }

    Ok(())
}

/// Check if a key exists in a hash table.
///
/// Signature: `key table -- boolean`
fn word_hash_table_exists(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let hash_table = interpreter.pop_as_hash_map()?;
    let key = interpreter.pop()?;

    if let Some(_) = hash_table.borrow().get(&key)
    {
        interpreter.push(true.to_value());
    }
    else
    {
        interpreter.push(false.to_value());
    }

    Ok(())
}

/// Merge a hash table into another.
///
/// Signature: `dest source -- dest`
fn word_hash_plus(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let source = interpreter.pop_as_hash_map()?;
    let dest = interpreter.pop_as_hash_map()?;

    dest.borrow_mut().extend(&source.borrow());

    interpreter.push(dest.to_value());

    Ok(())
}

/// Compare two hash tables and see if they are equal.
///
/// Signature: `a b -- boolean`
fn word_hash_compare(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop_as_hash_map()?;
    let a = interpreter.pop_as_hash_map()?;

    interpreter.push((a == b).to_value());

    Ok(())
}

/// Get the size of a hash table.
///
/// Signature: `table -- size`
fn word_hash_table_size(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let hash_table = interpreter.pop_as_hash_map()?;

    interpreter.push(hash_table.borrow().len().to_value());

    Ok(())
}

/// Iterate through a hash table and call a user word for each item.
///
/// Signature: `word-index hash -- `
///
/// Callback signature: `key value -- `
fn word_hash_table_iterate(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let hash_table = interpreter.pop_as_hash_map()?;
    let word_index = interpreter.pop_as_usize()?;

    for ( key, value ) in hash_table.borrow().iter()
    {
        interpreter.push(key.clone());
        interpreter.push(value.clone());

        interpreter.execute_word_index(&location_here!(), word_index)?;
    }

    Ok(())
}



/// Register the hash table words with the interpreter.
pub fn register_hash_table_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "{}.new", word_hash_table_new,
            "Create a new hash table.",
            " -- new_hash_table");

    add_native_word!(interpreter, "{}!", word_hash_table_insert,
        "Write a value to a given key in the table.",
        "value key table -- ");

    add_native_word!(interpreter, "{}@", word_hash_table_find,
        "Read a value from a given key in the table.",
        "key table -- value");

    add_native_word!(interpreter, "{}?", word_hash_table_exists,
        "Check if a given key exists in the table.",
        "key table -- bool");

    add_native_word!(interpreter, "{}.+", word_hash_plus,
        "Take two hashes and deep copy the contents from the second into the first.",
        "dest source -- dest");

    add_native_word!(interpreter, "{}.=", word_hash_compare,
        "Take two hashes and compare their contents.",
        "a b -- was-match");

    add_native_word!(interpreter, "{}.size@", word_hash_table_size,
        "Get the size of the hash table.",
        "table -- size");

    add_native_word!(interpreter, "{}.iterate", word_hash_table_iterate,
        "Iterate through a hash table and call a word for each item.",
        "word_index hash_table -- ");
}
