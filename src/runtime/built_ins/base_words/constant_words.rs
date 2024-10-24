
use crate::{ add_native_word,
             runtime::{ data_structures::value::{ ToValue, Value },
             interpreter::Interpreter } };



pub fn register_constant_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "none",
        |interpreter| { interpreter.push(&Value::None); Ok(()) },
        "Push the value of none onto the data stack.",
        " -- none");

    add_native_word!(interpreter, "true",
        |interpreter| { interpreter.push(&true.to_value()); Ok(()) },
        "Push the value true onto the data stack.",
        " -- true");

   add_native_word!(interpreter, "false",
       |interpreter| { interpreter.push(&false.to_value()); Ok(()) },
       "Push the value false onto the data stack.",
       " -- false");
}
