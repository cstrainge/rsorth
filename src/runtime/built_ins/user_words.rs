
use std::env::var;
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
                        error,
                        interpreter::Interpreter } };



fn word_user_env_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let name = interpreter.pop_as_string()?;
    let value =
        match var(name)
        {
            Ok(text) => text,
            Err(_)   => String::new()
        };

    interpreter.push(value.to_value());
    Ok(())
}

#[cfg(target_os = "windows")]
fn word_user_os_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push("Windows".to_string().to_value());
    Ok(())
}

#[cfg(target_os = "linux")]
fn word_user_os_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push("Linux".to_string().to_value());
    Ok(())
}

#[cfg(target_os = "macos")]
fn word_user_os_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push("macOS".to_string().to_value());
    Ok(())
}



pub fn register_user_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "user.env@", word_user_env_read,
        "Read an environment variable",
        "name -- value_or_empty");

    add_native_word!(interpreter, "user.os", word_user_os_read,
        "Get the name of the OS the script is running under.",
        " -- os_name");
}
