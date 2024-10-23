
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
                        error::{ self, script_error},
                        interpreter::Interpreter } };



fn word_dup(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(&value);
    interpreter.push(&value);

    Ok(())
}

fn word_drop(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let _ = interpreter.pop()?;

    Ok(())
}

fn word_swap(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let a = interpreter.pop()?;
    let b = interpreter.pop()?;

    interpreter.push(&a);
    interpreter.push(&b);

    Ok(())
}

fn word_over(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let a = interpreter.pop()?;
    let b = interpreter.pop()?;

    interpreter.push(&a);
    interpreter.push(&b);
    interpreter.push(&a);

    Ok(())
}

fn word_rot(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let c = interpreter.pop()?;
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    interpreter.push(&c);
    interpreter.push(&a);
    interpreter.push(&b);

    Ok(())
}

fn word_stack_depth(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push(&(interpreter.stack().len() as i64).to_value());

    Ok(())
}

fn word_pick(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let index = interpreter.pop_as_int()?;
    let count = interpreter.stack().len() as i64;

    if index < 0 || index >= count
    {
        script_error(interpreter, format!("Index {} out of range of stack size {}.",
                                          index,
                                          count))?;
    }

    let value = interpreter.pick(index as usize)?;
    interpreter.push(&value);

    Ok(())
}

fn word_push_to(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let index = interpreter.pop_as_int()?;
    let len = interpreter.stack().len() as i64;

    if index < 0 || index >= len
    {
        script_error(interpreter, format!("Index {} out of range of stack length {}.",
                                          index,
                                          len))?;
    }

    interpreter.push_to(index as usize)?;

    Ok(())
}



pub fn register_stack_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "dup", word_dup,
        "Duplicate the top value on the data stack.",
        "value -- value value");

    add_native_word!(interpreter, "drop", word_drop,
        "Discard the top value on the data stack.",
        "value -- ");

    add_native_word!(interpreter, "swap", word_swap,
        "Swap the top 2 values on the data stack.",
        "a b -- b a");

    add_native_word!(interpreter, "over", word_over,
        "Make a copy of the top value and place the copy under the second.",
        "a b -- b a b");

    add_native_word!(interpreter, "rot", word_rot,
        "Rotate the top 3 values on the stack.",
        "a b c -- c a b");

    add_native_word!(interpreter, "stack.depth", word_stack_depth,
        "Get the current depth of the stack.",
        " -- depth");

    add_native_word!(interpreter, "pick", word_pick,
        "Pick the value n locations down in the stack and push it on the top.",
        "n -- value");

    add_native_word!(interpreter, "push-to", word_push_to,
        "Pop the top value and push it back into the stack a position from the top.",
        "n -- <updated-stack>>");
}
