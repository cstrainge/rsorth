
// The code makes use of some of the newer features of Rust.  These features are not yet stable and
// require the nightly version of Rust to compile.  Because of this some of the code may not compile
// in a future version of Rust.  The features used are:
#![feature(let_chains)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]



/// Module for the managing source code and the generation of byte code.
#[macro_use]
mod lang;

/// Module for the runtime and the data structures used by the interpreter.  As well as the
/// interpreter itself.
mod runtime;



use std::env::{ args, current_exe, var };
use runtime::{ built_ins::{ base_words::register_base_words,
                            io_words::register_io_words,
                            terminal_words::register_terminal_words,
                            user_words::register_user_words,
                            ffi_words::register_ffi_words },
               data_structures::{ contextual_data::ContextualData,
                                  value::Value },
               error::{ self, ScriptError },
               interpreter::{ sorth_interpreter::SorthInterpreter,
                              CodeManagement,
                              Interpreter,
                              WordManagement } };



/// Get a directory path for the standard library.  This is either in the directory of the
/// executable or in a directory specified by the environment variable RSORTH_LIB_PATH.
fn std_lib_directory() -> error::Result<String>
{
    // Check for the environment variable first.
    if let Ok(lib_path) = var("RSORTH_LIB_PATH")
    {
        Ok(lib_path)
    }
    else
    {
        // The environment variable was not set.  Use the directory of the executable.
        match current_exe()
        {
            Ok(exe_path) =>
                {
                    if let Some(directory) = exe_path.parent()
                    {
                        match directory.to_str()
                        {
                            Some(dir_str) => Ok(dir_str.to_string()),
                            None => ScriptError::new_as_result(None,
                               "Executable directory path includes invalid characters.".to_string(),
                               None)
                        }
                    }
                    else
                    {
                        ScriptError::new_as_result(None,
                            "Could not get the directory of the running executable.".to_string(),
                            None)
                    }
                },

            Err(err) =>
                {
                    ScriptError::new_as_result(None,
                                      format!("Could not get the current executable path: {}", err),
                                      None)
                }
        }
    }
}


fn main() -> error::Result<()>
{
    // Create the core instance of the interpreter.  Then add the standard library's location to the
    // search path.
    let mut interpreter = SorthInterpreter::new();

    interpreter.add_search_path(&std_lib_directory()?)?;

    // Register the core standard library words.  These are all the words that are implemented in
    // Rust.
    register_base_words(&mut interpreter);
    register_io_words(&mut interpreter);
    register_terminal_words(&mut interpreter);
    register_user_words(&mut interpreter);
    register_ffi_words(&mut interpreter);

    // Find and process the standard library's main file.
    interpreter.process_source_file(&"std.f".to_string())?;

    // Mark the context as a "known good" state.  This is used to allow the user to reset the
    // interpreter to a solid state.
    interpreter.mark_context();

    // Gather the arguments passed to the script.  If there are arguments then the script to run is
    // the first argument and the rest are passed to the script as a list.
    let args: Vec<String> = args().collect();

    if args.len() >= 2
    {
        let script_args: Vec<&String> = args[2..].iter().collect();
        let script_args = Value::from(script_args);

        let handler = move |interpreter: &mut dyn Interpreter|
            {
                interpreter.push(script_args.clone());
                Ok(())
            };

        add_native_word!(&mut interpreter,
                        "sorth.args",
                        handler,
                        "List of command line arguments passed to the script.",
                        " -- argument_list");

        // Find and process the user's script file.
        let user_source = interpreter.find_file(&args[1])?;
        interpreter.process_source_file(&user_source)?;
    }
    else
    {
        // Else we start the REPL defined in the standard library.  If there isn't a REPL defined
        // then we just exit.
        interpreter.execute_word_named(&location_here!(), &"repl".to_string())?;
    }

    // Looks like everything went well.
    Ok(())
}
