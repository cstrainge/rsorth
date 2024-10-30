
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
                        error::{ self, script_error},
                        interpreter::Interpreter } };



/// Duplicate the top value on the data stack.
///
/// Signature: `value -- value value`
fn word_dup(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;

    interpreter.push(value.clone());
    interpreter.push(value);

    Ok(())
}

/// Drop the top value on the data stack.
///
/// Signature: `value -- `
fn word_drop(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let _ = interpreter.pop()?;

    Ok(())
}

/// Swap the top 2 values on the data stack.
///
/// Signature: `a b -- b a`
fn word_swap(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let a = interpreter.pop()?;
    let b = interpreter.pop()?;

    interpreter.push(a);
    interpreter.push(b);

    Ok(())
}

/// Make a copy of the second value and place the copy over and under the first item.
///
/// Signature: `a b -- b a b`
fn word_over(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    interpreter.push(b.clone());
    interpreter.push(a);
    interpreter.push(b);

    Ok(())
}

/// Rotate the top 3 values on the stack.
///
/// Signature: `a b c -- c a b`
fn word_rot(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let c = interpreter.pop()?;
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    interpreter.push(c);
    interpreter.push(a);
    interpreter.push(b);

    Ok(())
}

/// Get the depth of the data stack before calling this word.
///
/// Signature: ` -- depth`
fn word_stack_depth(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push(interpreter.stack().len().to_value());
    Ok(())
}

/// Get the current maximum depth of the data stack.
///
/// Signature: ` -- max-depth`
fn word_stack_max_depth(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.push(interpreter.stack_max_depth().to_value());
    Ok(())
}

/// Pick the value at the given index and push it on the top of the stack.
///
/// Signature: `index -- picked-value`
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
    interpreter.push(value);

    Ok(())
}

/// Pop the top value and push it back into the stack a position from the top.
///
/// Signature: `value -- <updated-stack>`
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



/// Register the stack manipulation words.
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
        "Get the depth of the stack before calling this word.",
        " -- depth");

    add_native_word!(interpreter, "stack.max-depth", word_stack_max_depth,
        "Get the current maximum depth of the stack.",
        " -- depth");

    add_native_word!(interpreter, "pick", word_pick,
        "Pick the value n locations down in the stack and push it on the top.",
        "n -- value");

    add_native_word!(interpreter, "push-to", word_push_to,
        "Pop the top value and push it back into the stack a position from the top.",
        "n -- <updated-stack>>");
}
