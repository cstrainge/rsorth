
#![allow(dead_code)]
#![feature(let_chains)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]



mod lang;
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



fn std_lib_directory() -> error::Result<String>
{
    if let Ok(lib_path) = var("RSORTH_LIB_PATH")
    {
        Ok(lib_path)
    }
    else
    {
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
    let mut interpreter = SorthInterpreter::new();

    interpreter.add_search_path(&std_lib_directory()?)?;

    register_base_words(&mut interpreter);
    register_io_words(&mut interpreter);
    register_terminal_words(&mut interpreter);
    register_user_words(&mut interpreter);
    register_ffi_words(&mut interpreter);

    // interpreter.process_source_file(&"std.f".to_string())?;

    interpreter.mark_context();

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

        let user_source = interpreter.find_file(&args[1])?;
        interpreter.process_source_file(&user_source)?;
    }
    else
    {
        interpreter.execute_word_named(&None, &"repl".to_string())?;
    }

    Ok(())
}
