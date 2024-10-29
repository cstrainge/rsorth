
use crate::{ add_native_immediate_word,
             add_native_word,
             lang::code::Op,
             location_here,
             runtime::{ data_structures::{ data_object::{ DataObject,
                                                          DataObjectDefinitionPtr,
                                                          DataObjectPtr },
                                           dictionary::{ WordInfo,
                                                         WordRuntime,
                                                         WordType,
                                                         WordVisibility },
                                           value::ToValue,
                                           value_hash::ValueHash },
                        error::{ self,
                                 script_error },
                        interpreter::Interpreter } };



fn word_word(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let token = interpreter.next_token()?;

    interpreter.push(token.to_value());
    Ok(())
}

fn get_word_info_definition(interpreter: &mut dyn Interpreter) -> DataObjectDefinitionPtr
{
    for definition in interpreter.structure_definitions()
    {
        if definition.borrow().name() == "sorth.word"
        {
            return definition.clone();
        }
    }

    panic!("Word info definition was not found.");
}

fn get_word_location_definition(interpreter: &mut dyn Interpreter) -> DataObjectDefinitionPtr
{
    for definition in interpreter.structure_definitions()
    {
        if definition.borrow().name() == "sorth.location"
        {
            return definition.clone();
        }
    }

    panic!("Word location definition was not found.");
}

fn convert_word_info(word: &WordInfo,
                     word_definition: &DataObjectDefinitionPtr,
                     location_definition: &DataObjectDefinitionPtr) -> DataObjectPtr
{
    let word_info_ptr = DataObject::new(word_definition);
    let location_ptr = DataObject::new(location_definition);

    {
        let mut word_info = word_info_ptr.borrow_mut();

        word_info.fields[0] = word.name.to_value();
        word_info.fields[1] = if word.runtime == WordRuntime::Immediate
                              {
                                  true.to_value()
                              }
                              else
                              {
                                  false.to_value()
                              };
        word_info.fields[2] = if word.word_type == WordType::Scripted
                              {
                                  true.to_value()
                              }
                              else
                              {
                                  false.to_value()
                              };
        word_info.fields[3] = if word.visibility == WordVisibility::Visible
                              {
                                  true.to_value()
                              }
                              else
                              {
                                  false.to_value()
                              };
        word_info.fields[4] = word.description.to_value();
        word_info.fields[5] = word.signature.to_value();
        word_info.fields[6] = word.handler_index.to_value();

        {
            let mut location = location_ptr.borrow_mut();

            location.fields[0] = word.location.path().to_value();
            location.fields[1] = word.location.line().to_value();
            location.fields[2] = word.location.column().to_value();
        }

        word_info.fields[7] = location_ptr.to_value();
    }

    word_info_ptr
}

fn word_get_word_table(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let word_definition = get_word_info_definition(interpreter);
    let location_definition = get_word_location_definition(interpreter);

    let dictionary = interpreter.dictionary().get_merged();
    let hash = ValueHash::new();

    for ( word, word_info ) in dictionary
    {
        hash.borrow_mut().insert(word.to_value(),
                                 convert_word_info(&word_info,
                                                   &word_definition,
                                                   &location_definition).to_value());
    }

    interpreter.push(hash.to_value());

    Ok(())
}

fn word_word_index(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let ( location, word ) = interpreter.next_token_word()?;

    if let Some(word_info) = interpreter.find_word(&word)
    {
        interpreter.insert_user_instruction(Some(location),
                                        Op::PushConstantValue(word_info.handler_index.to_value()))?;

        Ok(())
    }
    else
    {
        script_error(interpreter, format!("Word {} not found.", word))
    }
}

fn word_execute(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    if value.is_numeric()
    {
        let index = value.get_int_val();

        interpreter.execute_word_index(&location_here!(), index as usize)?;
    }
    else if value.is_stringable()
    {
        let word = value.get_string_val();

        interpreter.execute_word_named(&location_here!(), &word)?;
    }
    else
    {
        script_error(interpreter, format!("Value {} is not a valid word name or index.", value))?;
    }

    Ok(())
}

fn word_is_defined(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let word = interpreter.pop_as_string()?;
    let found = if let Some(_) = interpreter.find_word(&word) { true } else { false };

    interpreter.push(found.to_value());
    Ok(())
}

fn word_is_defined_im(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let ( _, word ) = interpreter.next_token_word()?;
    let found = if let Some(_) = interpreter.find_word(&word) { true } else { false };

    interpreter.push(found.to_value());
    Ok(())
}

fn word_is_undefined_im(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let ( _, word ) = interpreter.next_token_word()?;
    let not_found = if let Some(_) = interpreter.find_word(&word) { false } else { true };

    interpreter.push(not_found.to_value());
    Ok(())
}



pub fn register_word_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "word", word_word,
            "Get the next word in the token stream.",
            " -- next_word");

    add_native_word!(interpreter, "words.get{}", word_get_word_table,
        "Get a copy of the word table as it exists at time of calling.",
        " -- all_defined_words");

    add_native_immediate_word!(interpreter, "`", word_word_index,
        "Get the index of the next word.",
        " -- index");

    add_native_word!(interpreter, "execute", word_execute,
        "Execute a word name or index.",
        "word_name_or_index -- ???");

    add_native_word!(interpreter, "defined?", word_is_defined,
        "Is the given word defined?",
        " -- bool");

    add_native_immediate_word!(interpreter, "[defined?]", word_is_defined_im,
        "Evaluate at compile time, is the given word defined?",
        " -- bool");

    add_native_immediate_word!(interpreter, "[undefined?]", word_is_undefined_im,
        "Evaluate at compile time, is the given word not defined?",
        " -- bool");
}
