
use std::io::{ stdin, stdout, Write };
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
             error::{ self, script_error_str },
             interpreter::Interpreter } };



#[cfg(windows)]
/// Windows specific versions of the terminal words.
mod windows;

#[cfg(windows)]
use windows::{ init_win_console, word_term_raw_mode, word_term_size, word_term_key };



#[cfg(unix)]
/// Unix specific versions of the terminal words.
mod unix;

#[cfg(unix)]
use unix::{ word_term_raw_mode, word_term_size, word_term_key };



/// Flush the terminal buffers.
///
/// Signature: ` -- `
fn word_term_flush(_interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    stdout().flush()?;

    Ok(())
}

/// Read a line of text from the terminal.
///
/// Signature: ` -- string`
fn word_term_readline(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let mut line = String::new();

    stdin().read_line(&mut line)?;
    interpreter.push(line.trim_end_matches(&[ '\n', '\r' ]).to_string().to_value());

    Ok(())
}

/// Write a value as text to the console.
///
/// Signature: `value -- `
fn word_term_write(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    print!("{}", value);
    Ok(())
}

/// Is the given character printable in the terminal?
///
/// Signature: `character -- boolean`
fn word_term_is_printable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop_as_string()?;

    if value.chars().count() != 1
    {
        return script_error_str(interpreter, "Expected a single character.");
    }

    let character = value.chars().next().unwrap();
    let is_printable = !character.is_control();

    interpreter.push(is_printable.to_value());

    Ok(())
}



/// Register the terminal words with the interpreter.
pub fn register_terminal_words(interpreter: &mut dyn Interpreter)
{
    #[cfg(windows)]
    {
        init_win_console();
    }

    add_native_word!(interpreter, "term.raw_mode", word_term_raw_mode,
        "Enter or leave the terminal's 'raw' mode.",
        "bool -- ");

    add_native_word!(interpreter, "term.size@", word_term_size,
        "Return the number of characters in the rows and columns of the terminal.",
        " -- ");

    add_native_word!(interpreter, "term.key", word_term_key,
        "Read a keypress from the terminal.",
        " -- character");

    add_native_word!(interpreter, "term.flush", word_term_flush,
        "Flush the terminal buffers.",
        " -- ");

    add_native_word!(interpreter, "term.readline", word_term_readline,
        "Read a line of text from the terminal.",
        " -- string");

    add_native_word!(interpreter, "term.!", word_term_write,
        "Write a value to the console.",
        "value -- ");

    add_native_word!(interpreter, "term.is_printable?", word_term_is_printable,
        "Is the given character printable?",
        "character -- bool");
}
