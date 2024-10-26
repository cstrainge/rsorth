
use std::io::{ stdin, stdout, Write };
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
             error::{ self, script_error_str },
             interpreter::Interpreter } };



#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows::{ init_win_console, word_term_raw_mode, word_term_size, word_term_key };



#[cfg(unix)]
mod unix;

#[cfg(unix)]
use unix::{ word_term_raw_mode, word_term_size, word_term_key };



fn word_term_flush(_interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    stdout().flush()?;

    Ok(())
}

fn word_term_readline(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let mut line = String::new();

    stdin().read_line(&mut line)?;
    interpreter.push(line.trim_end_matches(&[ '\n', '\r' ]).to_string().to_value());

    Ok(())
}

fn word_term_write(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    print!("{}", value);
    Ok(())
}

fn word_term_is_printable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop_as_string()?;

    if value.chars().count() != 1
    {
        script_error_str(interpreter, "Expected a single character.")?;
    }

    let character = value.chars().next().unwrap();
    let result =    character.is_ascii_graphic()
                 || character == ' '
                 || character == '\t'
                 || character == '\n'
                 || character == '\n';

    interpreter.push(result.to_value());

    Ok(())
}



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
