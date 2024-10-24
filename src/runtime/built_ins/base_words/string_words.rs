
use std::sync::atomic::{ AtomicUsize, Ordering };
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
                        error::{ self,  script_error },
                        interpreter::Interpreter} };


fn word_hex(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    let number =
        if value.is_float()
        {
            let f_value = value.as_float(interpreter)?;
            f_value.to_bits() as i64
        }
        else if value.is_numeric()
        {
            value.get_int_val()
        }
        else
        {
            return script_error(interpreter, format!("Value {} is not a number.", value));
        };

    interpreter.push(format!("{:x}", number).to_value());
    Ok(())
}

fn word_unique_str(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    let index =  INDEX.fetch_add(1, Ordering::Relaxed);
    let unique_str = format!("unique-str-{:08x}", index);

    interpreter.push(unique_str.to_value());
    Ok(())
}



pub fn register_string_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "hex", word_hex,
        "Convert a number into a hex string.",
        "number -- hex_string");

    add_native_word!(interpreter, "unique_str", word_unique_str,
        "Generate a unique string and push it onto the data stack.",
        " -- string");
}
