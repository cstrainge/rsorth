
use crate::{ add_native_word,
             runtime::{ data_structures::value::{ ToValue,
                                                  Value },
             error::{ self,
                      script_error_str },
             interpreter::Interpreter } };



/// Helper function to handle string or numeric operations.  Handlers for each type of operation are
/// passed in as arguments.  The stack operations and value conversions are handled here.
fn string_or_numeric_op(interpreter: &mut dyn Interpreter,
                        fop: fn (&mut dyn Interpreter, f64, f64),
                        iop: fn (&mut dyn Interpreter, i64, i64),
                        sop: fn (&mut dyn Interpreter, String, String)) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    if Value::either_is_string(&a, &b)
    {
        let a = a.get_string_val();
        let b = b.get_string_val();

        sop(interpreter, a, b);
    }
    else if Value::either_is_float(&a, &b)
    {
        let a = a.get_float_val();
        let b = b.get_float_val();

        fop(interpreter, a, b);
    }
    else if Value::either_is_int(&a, &b)
    {
        let a = a.get_int_val();
        let b = b.get_int_val();

        iop(interpreter, a, b);
    }
    else
    {
        script_error_str(interpreter, "Value incompatible with numeric op.")?;
    }

    Ok(())
}

/// Helper function to handle math operations.  Handlers for int or floating point operations are
/// passed in as arguments.  The stack operations and value conversions are handled here.
fn math_op(interpreter: &mut dyn Interpreter,
           fop: fn (f64, f64) -> f64,
           iop: fn (i64, i64) -> i64)-> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;
    let mut result = Value::default();

    if Value::either_is_float(&a, &b)
    {
        let a = a.get_float_val();
        let b = b.get_float_val();

        result = fop(a, b).to_value();
    }
    else if Value::either_is_int(&a, &b)
    {
        let a = a.get_int_val();
        let b = b.get_int_val();

        result = iop(a, b).to_value();
    }
    else
    {
        script_error_str(interpreter, "Value incompatible with numeric op.")?;
    }

    interpreter.push(result);

    Ok(())
}

/// Helper function to handle logic operations.  The logic operation is passed in as an argument.
/// Tha stack operations and value conversions are handled here.
fn logic_op(interpreter: &mut dyn Interpreter,
            bop: fn (bool, bool) -> bool) -> error::Result<()>
{
    let b = interpreter.pop()?.get_bool_val();
    let a = interpreter.pop()?.get_bool_val();

    interpreter.push(bop(a, b).to_value());
    Ok(())
}

/// Helper function to handle bit logic operations.  The actual bit operation is passed in as an
/// argument.  The stack operations and value conversions are handled here.
fn logic_bit_op(interpreter: &mut dyn Interpreter,
                bop: fn (i64, i64) -> i64) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    if !Value::both_are_numeric(&a, &b)
    {
        script_error_str(interpreter, "Both bit logic operation values must be numeric.")?;
    }

    let a = a.get_int_val();
    let b = b.get_int_val();

    interpreter.push(bop(a, b).to_value());

    Ok(())
}



/// Add 2 numbers or strings together.
///
/// Signature: `a b -- result`
fn word_add(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    string_or_numeric_op(interpreter,
                         |i, a, b| { i.push((a + b).to_value()); },
                         |i, a, b| { i.push((a + b).to_value()); },
                         |i, a, b| { i.push((a + &b).to_value()); })
}

/// Subtract 2 numbers.
///
/// Signature: `a b -- result`
fn word_subtract(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    math_op(interpreter,
            |a, b| { a - b },
            |a, b| { a - b })
}

/// Multiply 2 numbers.
///
/// Signature: `a b -- result`
fn word_multiply(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    math_op(interpreter,
            |a, b| { a * b },
            |a, b| { a * b })
}

/// Divide 2 numbers.
///
/// Signature: `a b -- result`
fn word_divide(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    math_op(interpreter,
            |a, b| { a / b },
            |a, b| { a / b })
}

/// Mod 2 numbers.
///
/// Signature: `a b -- result`
fn word_mod(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    math_op(interpreter,
            |a, b| { a % b },
            |a, b| { a % b })
}

/// Logically and 2 boolean values.
///
/// Signature: `a b -- result`
fn word_logic_and(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    logic_op(interpreter, |a, b| { a && b })
}

/// Logically or 2 boolean values.
///
/// Signature: `a b -- result`
fn word_logic_or(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    logic_op(interpreter, |a, b| { a || b })
}

/// Logically invert a boolean value.
///
/// Signature: `a -- a'`
fn word_logic_not(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let a = interpreter.pop_as_bool()?;

    interpreter.push({!a}.to_value());
    Ok(())
}

/// Bitwise AND two numbers together.
///
/// Signature: `a b -- result`
fn word_bit_and(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    logic_bit_op(interpreter, |a, b| { a & b })
}

/// Bitwise OR two numbers together.
///
/// Signature: `a b -- result`
fn word_bit_or(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    logic_bit_op(interpreter, |a, b| { a | b })
}

/// Bitwise XOR two numbers together.
///
/// Signature: `a b -- result`
fn word_bit_xor(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    logic_bit_op(interpreter, |a, b| { a ^ b })
}

/// Bitwise NOT a number.
///
/// Signature: `a -- !a`
fn word_bit_not(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let a = interpreter.pop_as_int()?;

    interpreter.push((!a).to_value());
    Ok(())
}

/// Shift a number of bits to the left.
///
/// Signature: `a count -- result`
fn word_bit_left_shift(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    logic_bit_op(interpreter, |value, amount| { value << amount })
}

/// Shift a number of bits to the right.
///
/// Signature: `a count -- result`
fn word_bit_right_shift(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    logic_bit_op(interpreter, |value, amount| { value >> amount })
}

/// Are 2 values equal?
///
/// Signature: `a b -- boolean`
fn word_equal(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;
    let result = a == b;

    interpreter.push(result.to_value());
    Ok(())
}

/// Is one value greater or equal to another?
///
/// Signature: `a b -- boolean`
fn word_greater_equal(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    interpreter.push((a >= b).to_value());

    Ok(())
}

/// Is one value lesser or equal to another?
///
/// Signature: `a b -- boolean`
fn word_less_equal(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    interpreter.push((a <= b).to_value());

    Ok(())
}

/// Is one value greater than another?
///
/// Signature: `a b -- boolean`
fn word_greater(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    interpreter.push((a > b).to_value());

    Ok(())
}

/// Is one value less than another?
///
/// Signature: `a b -- boolean`
fn word_less(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let b = interpreter.pop()?;
    let a = interpreter.pop()?;

    interpreter.push((a < b).to_value());

    Ok(())
}



/// Regester all of the math, logic, bit, and equality words.
pub fn register_math_logic_and_bit_words(interpreter: &mut dyn Interpreter)
{
    // Math ops.
    add_native_word!(interpreter, "+", word_add,
        "Add 2 numbers or strings together.",
        "a b -- result");

    add_native_word!(interpreter, "-", word_subtract,
        "Subtract 2 numbers.",
        "a b -- result");

    add_native_word!(interpreter, "*", word_multiply,
        "Multiply 2 numbers.",
        "a b -- result");

    add_native_word!(interpreter, "/", word_divide,
        "Divide 2 numbers.",
        "a b -- result");

    add_native_word!(interpreter, "%", word_mod,
        "Mod 2 numbers.",
        "a b -- result");


    // Logical words.
    add_native_word!(interpreter, "&&", word_logic_and,
        "Logically compare 2 values.",
        "a b -- bool");

    add_native_word!(interpreter, "||", word_logic_or,
        "Logically compare 2 values.",
        "a b -- bool");

    add_native_word!(interpreter, "'", word_logic_not,
        "Logically invert a boolean value.",
        "bool -- bool");


    // Bitwise operator words.
    add_native_word!(interpreter, "&", word_bit_and,
        "Bitwise AND two numbers together.",
        "a b -- result");

    add_native_word!(interpreter, "|", word_bit_or,
        "Bitwise OR two numbers together.",
        "a b -- result");

    add_native_word!(interpreter, "^", word_bit_xor,
        "Bitwise XOR two numbers together.",
        "a b -- result");

    add_native_word!(interpreter, "~", word_bit_not,
        "Bitwise NOT a number.",
        "number -- result");

    add_native_word!(interpreter, "<<", word_bit_left_shift,
        "Shift a numbers bits to the left.",
        "value amount -- result");

    add_native_word!(interpreter, ">>", word_bit_right_shift,
        "Shift a numbers bits to the right.",
        "value amount -- result");


    // Equality words.
    add_native_word!(interpreter, "=", word_equal,
        "Are 2 values equal?",
        "a b -- bool");

    add_native_word!(interpreter, ">=", word_greater_equal,
        "Is one value greater or equal to another?",
        "a b -- bool");

    add_native_word!(interpreter, "<=", word_less_equal,
        "Is one value less than or equal to another?",
        "a b -- bool");

    add_native_word!(interpreter, ">", word_greater,
        "Is one value greater than another?",
        "a b -- bool");

    add_native_word!(interpreter, "<", word_less,
        "Is one value less than another?",
        "a b -- bool");
}
