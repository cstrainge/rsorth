
use sysinfo::System;
use crate::{ add_native_immediate_word,
             add_native_word,
             runtime::{ data_structures::value::{ ToValue, Value},
             error::{ self, script_error, script_error_str },
             interpreter::Interpreter } };



fn word_reset(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.reset()
}

fn word_include(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let file = interpreter.pop_as_string()?;
    interpreter.process_source_file(&file)
}

fn word_include_im(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let file = interpreter.next_token_text()?;
    interpreter.process_source_file(&file)
}

fn word_print_stack(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    println!("Depth: {}", interpreter.stack().len());

    for value in interpreter.stack()
    {
        if value.is_string()
        {
            println!("{}", Value::stringify(&value.to_string()));
        }
        else
        {
            println!("{}", value);
        }
    }

    Ok(())
}

fn word_print_dictionary(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    print!("{}", interpreter.dictionary());
    Ok(())
}

fn word_thread_show(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error(interpreter, format!("Word {} not implemented yet.", "word_thread_show"))
}

fn word_print_structures(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error(interpreter, format!("Word {} not implemented yet.", "word_print_structures"))
}

fn word_sorth_version(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push(&env!("CARGO_PKG_VERSION").to_string().to_value());
    Ok(())
}

fn word_sorth_search_path(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push(&Value::from(interpreter.search_paths()));
    Ok(())
}

fn word_sorth_find_file(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let file = interpreter.pop_as_string()?;
    let full_path = interpreter.find_file(&file)?;

    interpreter.push(&full_path.to_value());

    Ok(())
}

fn word_sorth_memory(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let mut system = System::new();

    system.refresh_all();

    let pid = sysinfo::get_current_pid();

    if let Ok(pid) = pid
    {
        if let Some(process) = system.process(pid)
        {
            interpreter.push(&(process.memory() as i64).to_value());
        }
        else
        {
            script_error_str(interpreter, "Could not read process memory information.")?;
        }
    }
    else
    {
        script_error_str(interpreter, "Could not read process pid.")?;
    }

    Ok(())
}

fn word_throw(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let message = interpreter.pop_as_string()?;
    script_error(interpreter, message)
}

fn word_thread_new(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error(interpreter, format!("Word {} not implemented yet.", "word_thread_new"))
}

fn word_thread_push_to(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error(interpreter, format!("Word {} not implemented yet.", "word_thread_push_to"))
}

fn word_thread_pop_from(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error(interpreter, format!("Word {} not implemented yet.", "word_thread_pop_from"))
}

fn word_thread_push(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error(interpreter, format!("Word {} not implemented yet.", "word_thread_push"))
}

fn word_thread_pop(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error(interpreter, format!("Word {} not implemented yet.", "word_thread_pop"))
}



pub fn register_sorth_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "reset", word_reset,
        "Reset the interpreter to it's default state.",
        " -- ");

    add_native_word!(interpreter, "include", word_include,
        "Include and execute another source file.",
        "source_path -- ");

    add_native_immediate_word!(interpreter, "[include]", word_include_im,
        "Include and execute another source file.",
        "[include] file/to/include.f");

    add_native_word!(interpreter, ".s", word_print_stack,
        "Print out the data stack without changing it.",
        " -- ");

    add_native_word!(interpreter, ".w", word_print_dictionary,
        "Print out the current word dictionary.",
        " -- ");

    add_native_word!(interpreter, ".t", word_thread_show,
            "Print out the list of interpreter threads.",
            " -- ");

    add_native_word!(interpreter, ".#", word_print_structures,
        "Print out the currently available data structures.",
        " -- ");

    add_native_word!(interpreter, "sorth.version", word_sorth_version,
        "Get the current version of the interpreter.",
        " -- version_string");

    add_native_word!(interpreter, "sorth.search-path", word_sorth_search_path,
        "Get the search path being used by the interpreter.",
        " -- search-paths");

    add_native_word!(interpreter, "sorth.find-file", word_sorth_find_file,
        "Search for a file within the given search paths.",
        " -- full-file-path");

    add_native_word!(interpreter, "sorth.memory", word_sorth_memory,
        "Get the size of the process's working set.",
        " -- memory-size");

    add_native_word!(interpreter, "throw", word_throw,
        "Throw an exception with the given message.",
        "message -- ");

    add_native_word!(interpreter, "thread.new", word_thread_new,
        "Create a new thread and run the specified word and return the new thread id.",
        "word-index -- thread-id");

    add_native_word!(interpreter, "thread.push-to", word_thread_push_to,
        "Push the top value to another thread's input stack.",
        "value thread-id -- ");

    add_native_word!(interpreter, "thread.pop-from", word_thread_pop_from,
        "Pop a value off of the threads input queue, block if there's nothing available.",
        "thread-id -- input-value");

    add_native_word!(interpreter, "thread.push", word_thread_push,
        "Push the top value onto the thread's output queue.",
        "output-value -- ");

    add_native_word!(interpreter, "thread.pop", word_thread_pop,
        "Pop from another thread's output stack and push onto the local data stack.",
        " -- value");
}
