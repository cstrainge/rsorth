
use crate::{ add_native_word,
             runtime::{ data_structures::{ value::ToValue,
                                           value_vec::{ ValueVec,
                                                        ValueVecPtr } },
                                           error::{ self,
                                                    script_error,
                                                    script_error_str },
                        interpreter::Interpreter } };



fn check_bounds(interpreter: &mut dyn Interpreter,
                array: &ValueVecPtr,
                index: &usize) -> error::Result<()>
{
    if *index > array.borrow().len()
    {
        script_error(interpreter, format!("Index {} is out of bounds for array of size {}.",
                                          index,
                                          array.borrow().len()))?;
    }

    Ok(())
}



fn word_array_new(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let size = interpreter.pop_as_usize()?;
    let array = ValueVec::new(size);

    interpreter.push(array.to_value());
    Ok(())
}

fn word_array_size(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;

    interpreter.push((array.borrow().len() as i64).to_value());
    Ok(())
}

fn word_array_write_index(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;
    let index = interpreter.pop_as_usize()?;
    let value = interpreter.pop()?;

    check_bounds(interpreter, &array, &index)?;

    array.borrow_mut()[index] = value;

    Ok(())
}

fn word_array_read_index(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;
    let index = interpreter.pop_as_usize()?;

    check_bounds(interpreter, &array, &index)?;

    interpreter.push(array.borrow()[index].clone());

    Ok(())
}

fn word_array_resize(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;
    let new_size = interpreter.pop_as_usize()?;

    array.borrow_mut().resize(new_size);

    Ok(())
}

fn word_array_insert(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;
    let index = interpreter.pop_as_usize()?;
    let value = interpreter.pop()?;

    check_bounds(interpreter, &array, &index)?;

    array.borrow_mut().insert(index, value);

    Ok(())
}

fn word_array_delete(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;
    let index = interpreter.pop_as_usize()?;

    check_bounds(interpreter, &array, &index)?;

    array.borrow_mut().remove(index);

    Ok(())
}

fn word_array_plus(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let source = interpreter.pop_as_array()?;
    let dest = interpreter.pop_as_array()?;

    dest.borrow_mut().extend(&source.borrow());

    interpreter.push(dest.to_value());

    Ok(())
}

fn word_array_compare(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop_as_array()?;
    let a = interpreter.pop_as_array()?;

    interpreter.push((a == b).to_value());

    Ok(())
}

fn word_push_front(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;
    let value = interpreter.pop()?;

    array.borrow_mut().push_front(value);

    Ok(())
}

fn word_push_back(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;
    let value = interpreter.pop()?;

    array.borrow_mut().push_back(value);

    Ok(())
}

fn word_pop_front(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;

    if let Some(value) = array.borrow_mut().pop_front()
    {
        interpreter.push(value);
    }
    else
    {
        script_error_str(interpreter, "[].pop_front from an empty array.")?;
    }

    Ok(())
}

fn word_pop_back(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let array = interpreter.pop_as_array()?;

    if let Some(value) = array.borrow_mut().pop_back()
    {
        interpreter.push(value);
    }
    else
    {
        script_error_str(interpreter, "[].pop_back from an empty array.")?;
    }

    Ok(())
}



pub fn register_array_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "[].new", word_array_new,
        "Create a new array with the given default size.",
        " -- array");

    add_native_word!(interpreter, "[].size@", word_array_size,
        "Read the size of the array object.",
        "array -- size");

    add_native_word!(interpreter, "[]!", word_array_write_index,
        "Write to a value in the array.",
        "value index array -- ");

    add_native_word!(interpreter, "[]@", word_array_read_index,
        "Read a value from the array.",
        "index array -- value");

    add_native_word!(interpreter, "[].size!", word_array_resize,
        "Grow or shrink the array to the new size.",
        "array -- size");

    add_native_word!(interpreter, "[].insert", word_array_insert,
        "Grow an array by inserting a value at the given location.",
        "value index array -- ");

    add_native_word!(interpreter, "[].delete", word_array_delete,
        "Shrink an array by removing the value at the given location.",
        "index array -- ");

    add_native_word!(interpreter, "[].+", word_array_plus,
        "Take two arrays and deep copy the contents from the second into the first.",
        "dest source -- dest");

    add_native_word!(interpreter, "[].=", word_array_compare,
        "Take two arrays and compare the contents to each other.",
        "dest source -- dest");

    add_native_word!(interpreter, "[].push_front!", word_push_front,
        "Push a value to the front of an array.",
        "value array -- ");

    add_native_word!(interpreter, "[].push_back!", word_push_back,
        "Push a value to the end of an array.",
        "value array -- ");

    add_native_word!(interpreter, "[].pop_front!", word_pop_front,
        "Pop a value from the front of an array.",
        "array -- value");

    add_native_word!(interpreter, "[].pop_back!", word_pop_back,
        "Pop a value from the back of an array.",
        "array -- value");
}
