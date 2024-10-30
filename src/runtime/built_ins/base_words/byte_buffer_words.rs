
use crate::{ add_native_word,
             runtime::{ data_structures::{ byte_buffer::{ Buffer,
                                                          ByteBuffer,
                                                          ByteBufferPtr },
                                           value::ToValue },
                        error::{ self,
                                 script_error },
                        interpreter::Interpreter } };



/// Make sure the next read or write will not violate the bounds of the buffer.
fn check_buffer_index(interpreter: &mut dyn Interpreter,
                      buffer_ptr: &ByteBufferPtr,
                      byte_size: usize) -> error::Result<()>
{
    if buffer_ptr.borrow().position() + byte_size > buffer_ptr.borrow().len()
    {
        script_error(interpreter,
               format!("Writing a value size {} at a position {} would exceed the buffer size {}.",
                       byte_size,
                       buffer_ptr.borrow().position(),
                       buffer_ptr.borrow().len()))?;
    }

    Ok(())
}



/// Create a new ByteBuffer of the given size.
///
/// Signature: `size -- buffer`
fn word_buffer_new(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let size = interpreter.pop_as_usize()?;
    let buffer = ByteBuffer::new(size);

    interpreter.push(buffer.to_value());

    Ok(())
}

/// Get the size of a ByteBuffer.
///
/// Signature: `buffer -- size`
fn word_buffer_size(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let buffer = interpreter.pop_as_byte_buffer()?;

    interpreter.push(buffer.borrow().len().to_value());

    Ok(())
}

/// Resize a given ByteBuffer, either growing or shrinking it.  If the buffer is grown it is padded
/// with 0s.
///
/// Signature: `size buffer -- `
fn word_buffer_resize(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let size = interpreter.pop_as_usize()?;
    let buffer = interpreter.pop_as_byte_buffer()?;

    buffer.borrow_mut().resize(size);

    Ok(())
}

/// Write an integer of a given size to the buffer.  The only valid sizes are 1, 2, 4, and 8 bytes.
///
/// Signature: `value buffer byte-size -- `
fn word_buffer_write_int(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let byte_size = interpreter.pop_as_usize()?;
    let buffer_ptr = interpreter.pop_as_byte_buffer()?;
    let value = interpreter.pop_as_int()?;

    check_buffer_index(interpreter, &buffer_ptr, byte_size)?;

    if    (byte_size != 1)
       && (byte_size != 2)
       && (byte_size != 4)
       && (byte_size != 8)
    {
        script_error(interpreter, format!("Invalid byte size {} for integer value.", byte_size))?;
    }

    buffer_ptr.borrow_mut().write_int(byte_size, value);

    Ok(())
}

/// Read an integer of a given size from the buffer.  The only valid sizes are 1, 2, 4, and 8 bytes.
/// If the value is signed and negative the value will be sign extended.
///
/// Signature: `buffer byte-size is-signed -- value`
fn word_buffer_read_int(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let is_signed = interpreter.pop_as_bool()?;
    let byte_size = interpreter.pop_as_usize()?;
    let buffer_ptr = interpreter.pop_as_byte_buffer()?;

    check_buffer_index(interpreter, &buffer_ptr, byte_size)?;

    if    (byte_size != 1)
       && (byte_size != 2)
       && (byte_size != 4)
       && (byte_size != 8)
    {
        script_error(interpreter, format!("Invalid byte size {} for integer value.", byte_size))?;
    }

    let value = buffer_ptr.borrow_mut().read_int(byte_size, is_signed);
    interpreter.push(value.to_value());

    Ok(())
}

/// Write a floating point value of a given size to the buffer.  The only valid sizes are 4 and 8
/// bytes.
///
/// Signature: `value buffer byte-size -- `
fn word_buffer_write_float(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let byte_size = interpreter.pop_as_usize()?;
    let buffer_ptr = interpreter.pop_as_byte_buffer()?;
    let value = interpreter.pop_as_float()?;

    if    (byte_size != 4)
       && (byte_size != 8)
    {
        script_error(interpreter, format!("Invalid byte size {} for floating point value.",
                                          byte_size))?;
    }

    buffer_ptr.borrow_mut().write_float(byte_size, value);

    Ok(())
}

/// Read a floating point value of a given size from the buffer.  The only valid sizes are 4 and 8
/// bytes.
///
/// Signature: `buffer byte-size -- value`
fn word_buffer_read_float(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let byte_size = interpreter.pop_as_usize()?;
    let buffer_ptr = interpreter.pop_as_byte_buffer()?;

    check_buffer_index(interpreter, &buffer_ptr, byte_size)?;

    if    (byte_size != 4)
       && (byte_size != 8)
    {
        script_error(interpreter, format!("Invalid byte size {} for floating point value.",
                                          byte_size))?;
    }

    let value = buffer_ptr.borrow_mut().read_float(byte_size);
    interpreter.push(value.to_value());

    Ok(())
}

/// Write a string of a given size to the buffer.  If the string is too short it will be padded with
/// 0s in the buffer.  If it is larger than the size it will be truncated.
///
/// Signature: `value buffer byte-size -- `
fn word_buffer_write_string(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let byte_size = interpreter.pop_as_usize()?;
    let buffer_ptr = interpreter.pop_as_byte_buffer()?;
    let value = interpreter.pop_as_string()?;

    check_buffer_index(interpreter, &buffer_ptr, byte_size)?;

    buffer_ptr.borrow_mut().write_string(byte_size, &value);

    Ok(())
}

/// Read a string of a given size from the buffer.  If the actual string data is shorter than the
/// size it will be truncated.  It will be treated as a null-terminated string.
///
/// Signature: `buffer byte-size -- value`
fn word_buffer_read_string(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let byte_size = interpreter.pop_as_usize()?;
    let buffer_ptr = interpreter.pop_as_byte_buffer()?;

    check_buffer_index(interpreter, &buffer_ptr, byte_size)?;

    let value = buffer_ptr.borrow_mut().read_string(byte_size);
    interpreter.push(value.to_value());

    Ok(())
}

/// Set the position of the cursor in the buffer.  This is the position that the next read or write
/// will occur at.
///
/// Signature: `position buffer -- `
fn word_buffer_set_position(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let buffer = interpreter.pop_as_byte_buffer()?;
    let position = interpreter.pop_as_usize()?;

    if position > buffer.borrow().len()
    {
        script_error(interpreter, format!("Setting buffer position {} beyond buffer size {}.",
                                          position,
                                          buffer.borrow().len()))?;
    }

    buffer.borrow_mut().set_position(position);

    Ok(())
}

/// Get the current position of the read/write cursor in the buffer.
///
/// Signature: `buffer -- position`
fn word_buffer_get_position(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let buffer = interpreter.pop_as_byte_buffer()?;
    let position = buffer.borrow().position();

    interpreter.push(position.to_value());

    Ok(())
}



/// Register all of the byte buffer words with the given interpreter.
pub fn register_byte_buffer_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "buffer.new", word_buffer_new,
        "Create a new byte buffer.",
        "size -- buffer");

    add_native_word!(interpreter, "buffer.size@", word_buffer_size,
        "Get the size of a byte buffer.",
        " -- size");

    add_native_word!(interpreter, "buffer.size!", word_buffer_resize,
        "Resize an existing byte buffer.",
        "size buffer -- ");

    add_native_word!(interpreter, "buffer.int!", word_buffer_write_int,
        "Write an integer of a given size to the buffer.",
        "value buffer byte_size -- ");

    add_native_word!(interpreter, "buffer.int@", word_buffer_read_int,
        "Read an integer of a given size from the buffer.",
        "buffer byte_size is_signed -- value");

    add_native_word!(interpreter, "buffer.float!", word_buffer_write_float,
        "Write a float of a given size to the buffer.",
        "value buffer byte_size -- ");

    add_native_word!(interpreter, "buffer.float@", word_buffer_read_float,
        "read a float of a given size from the buffer.",
        "buffer byte_size -- value");

    add_native_word!(interpreter, "buffer.string!", word_buffer_write_string,
        "Write a string of given size to the buffer.  Padded with 0s if needed.",
        "value buffer size -- ");

    add_native_word!(interpreter, "buffer.string@", word_buffer_read_string,
        "Read a string of a given max size from the buffer.",
        "size buffer -- value");

    add_native_word!(interpreter, "buffer.position!", word_buffer_set_position,
        "Set the position of the buffer pointer.",
        "position buffer -- ");

    add_native_word!(interpreter, "buffer.position@", word_buffer_get_position,
        "Get the position of the buffer pointer.",
        "buffer -- position");
}
