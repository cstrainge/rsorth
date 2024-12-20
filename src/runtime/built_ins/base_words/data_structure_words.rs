
use crate::{ add_native_word,
             location_here,
             runtime::{ data_structures::{ data_object::{ DataObject,
                                                          DataObjectDefinition,
                                                          DataObjectPtr },
                                           value::ToValue,
                                           value_vec::ValueVec },
                        error::{ self,
                                 script_error,
                                 script_error_str },
                        interpreter::Interpreter } };



/// Validate that the index is within the bounds of the structure.
fn check_index(interpreter: &mut dyn Interpreter,
               data_ptr: &DataObjectPtr,
               index: &usize) -> error::Result<()>
{
    if *index >= data_ptr.borrow().fields.len()
    {
        script_error(interpreter, format!("Field index {} is out of range for structure {}.",
                                          index,
                                          data_ptr.borrow().definition_ptr.borrow().name()))?;
    }

    Ok(())
}



/// Create a new structure definition and add it's helper words to the interpreter.  This word is
/// not intended to be called directly user code.  Instead a Forth word is defined that handles
/// the structure creation syntax and will call this word.
///
/// Signature: `[defaults] name field-names is-hidden found-initializers? -- `
fn word_data_definition(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let location = interpreter.current_location().clone();
    let found_initializers = interpreter.pop_as_bool()?;
    let is_hidden = interpreter.pop_as_bool()?;
    let fields = interpreter.pop_as_array()?;
    let name = interpreter.pop_as_string()?;
    let defaults =
        if found_initializers
        {
            interpreter.pop_as_array()?
        }
        else
        {
            ValueVec::new(fields.borrow().len())
        };

    let mut field_names = Vec::with_capacity(fields.borrow().len());

    for field in fields.borrow().iter()
    {
        if !field.is_stringable()
        {
            script_error_str(interpreter, "Field names must be strings.")?;
        }

        field_names.push(field.get_string_val().clone());
    }

    let defaults = defaults.borrow().iter().map(|value| value.clone()).collect();

    let definition_ptr = DataObjectDefinition::new(interpreter,
                                                   name,
                                                   field_names,
                                                   defaults,
                                                   is_hidden.clone());

    DataObjectDefinition::create_data_definition_words(interpreter,
                                                       location,
                                                       definition_ptr,
                                                       is_hidden);

    Ok(())
}

/// Read a field from a structure.
///
/// Signature: `structure index -- value`
fn word_read_field(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let index = interpreter.pop_as_usize()?;
    let data_ptr = interpreter.pop_as_data_object()?;

    check_index(interpreter, &data_ptr, &index)?;

    interpreter.push(data_ptr.borrow().fields[index].clone());

    Ok(())
}

/// Write a field to a structure.
///
/// Signature: `value structure index -- `
fn word_write_field(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let index = interpreter.pop_as_usize()?;
    let data_ptr = interpreter.pop_as_data_object()?;
    let value = interpreter.pop()?;

    check_index(interpreter, &data_ptr, &index)?;

    data_ptr.borrow_mut().fields[index] = value;

    Ok(())
}

/// Iterate through the fields of a structure and call a word for each field.
///
/// Signature: `word-index structure -- `
///
/// The callback signature: `field-name field-value -- `
fn word_structure_iterate(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let data_ptr = interpreter.pop_as_data_object()?;
    let word_index = interpreter.pop_as_usize()?;

    for index in 0..data_ptr.borrow().fields.len()
    {
        interpreter.push(data_ptr.borrow().definition_ptr.borrow().field_names()[index].to_value());
        interpreter.push(data_ptr.borrow().fields[index].clone());

        interpreter.execute_word_index(&location_here!(), word_index)?;
    }

    Ok(())
}

/// Check if a field exists in a structure.
///
/// Signature: `field-name structure -- boolean`
fn word_structure_field_exists(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let data_ptr = interpreter.pop_as_data_object()?;
    let field_name = interpreter.pop_as_string()?;
    let index = data_ptr.borrow()
                        .definition_ptr.borrow()
                        .field_names()
                        .iter()
                        .position(|found| *found == field_name);

    let found = index.is_some();

    interpreter.push(found.to_value());

    Ok(())
}

/// Compare two structures to see if they are logically the same.
///
/// Signature: `a b -- boolean`
fn word_structure_compare(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop_as_data_object()?;
    let a = interpreter.pop_as_data_object()?;

    interpreter.push((a == b).to_value());

    Ok(())
}



/// Register the structs `sorth.word` and `sorth.location` with the interpreter.
fn register_word_info_struct(interpreter: &mut dyn Interpreter)
{
    let location = DataObjectDefinition::new(interpreter,
                                             "sorth.location".to_string(),
                                             vec![ "path".to_string(),
                                                   "line".to_string(),
                                                   "column".to_string() ],
                                             vec![ "".to_string().to_value(),
                                                   1usize.to_value(),
                                                   1usize.to_value() ],
                                             true);

    let default_location = DataObject::new(&location);

    DataObjectDefinition::create_data_definition_words(interpreter,
                                                       Some(location_here!()),
                                                       location,
                                                       true);

    let word_info = DataObjectDefinition::new(interpreter,
                                              "sorth.word".to_string(),
                                              vec![ "name".to_string(),
                                                    "is_immediate".to_string(),
                                                    "is_scripted".to_string(),
                                                    "is_visible".to_string(),
                                                    "description".to_string(),
                                                    "signature".to_string(),
                                                    "handler_index".to_string(),
                                                    "location".to_string() ],
                                              vec![ "".to_string().to_value(),
                                                    false.to_value(),
                                                    false.to_value(),
                                                    false.to_value(),
                                                    "".to_string().to_value(),
                                                    "".to_string().to_value(),
                                                    0usize.to_value(),
                                                    default_location.to_value() ],
                                              true);

    DataObjectDefinition::create_data_definition_words(interpreter,
                                                       Some(location_here!()),
                                                       word_info,
                                                       true);
}



/// Register all of the structure words with teh interpreter.
pub fn register_data_structure_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "#", word_data_definition,
        "Beginning of a structure definition.",
        " -- ");

    add_native_word!(interpreter, "#@", word_read_field,
        "Read a field from a structure.",
        "field_index structure -- value");

    add_native_word!(interpreter, "#!", word_write_field,
        "Write to a field of a structure.",
        "value field_index structure -- ");

    add_native_word!(interpreter, "#.iterate", word_structure_iterate,
        "Call an iterator for each member of a structure.",
        "word_or_index -- ");

    add_native_word!(interpreter, "#.field-exists?", word_structure_field_exists,
        "Check if the named structure field exits.",
        "field_name structure -- boolean");

    add_native_word!(interpreter, "#.=", word_structure_compare,
        "Check if two structures are the same.",
        "a b -- boolean");

    register_word_info_struct(interpreter);
}
