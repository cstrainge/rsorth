
use crate::{ add_native_word,
             lang::{ code::{ Instruction, Op },
                     compilation::{process_token, InsertionLocation},
                     tokenizing::Token },
             runtime::{ data_structures::value::ToValue,
                        error::{self, script_error},
                        interpreter::Interpreter } };



/// Insert an instruction op into the current byte-code stream.
fn insert_user_instruction(interpreter: &mut dyn Interpreter, op: Op) -> error::Result<()>
{
    let instruction = Instruction::new(None, op);
    interpreter.context_mut().push_instruction(instruction)
}



/// Push a define variable instruction into the byte-code stream.
///
/// Signature: `name -- `
fn word_op_def_variable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::DefVariable(value))
}

/// Push a define constant instruction into the byte-code stream.
///
/// Signature: `value -- `
fn word_op_def_constant(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::DefConstant(value))
}

/// Push a read variable instruction into the byte-code stream.
///
/// Signature: ` -- `
fn word_op_read_variable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::ReadVariable)

}

/// Push a write variable instruction into the byte-code stream.
///
/// Signature: ` -- `
fn word_op_write_variable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::WriteVariable)

}

/// Push an execute instruction into the byte-code stream.
///
/// Signature: `name-or-index -- `
fn word_op_execute(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::Execute(value))
}

/// Push a push constant value instruction into the byte-code stream.
///
/// Signature: `value -- `
fn word_op_push_constant_value(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::PushConstantValue(value))
}

/// Push a mark loop exit instruction into the byte-code stream.
///
/// Signature: `jump-label -- `
fn word_mark_loop_exit(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::MarkLoopExit(value))

}

/// Push an unmark loop exit instruction into the byte-code stream.
///
/// Signature: ` -- `
fn word_unmark_loop_exit(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::UnmarkLoopExit)
}

/// Push a mark catch instruction into the byte-code stream.
///
/// Signature: `jump-label -- `
fn word_op_mark_catch(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::MarkCatch(value))
}

/// Push an unmark catch instruction into the byte-code stream.
///
/// Signature: ` -- `
fn word_op_unmark_catch(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::UnmarkCatch)
}

/// Push a jump instruction into the byte-code stream.
///
/// Signature: `jump-label -- `
fn word_op_jump(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::Jump(value))
}

/// Push a jump if zero instruction into the byte-code stream.
///
/// Signature: `jump-label -- `
fn word_op_jump_if_zero(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::JumpIfZero(value))
}

/// Push a jump if not zero instruction into the byte-code stream.
///
/// Signature: `jump-label -- `
fn word_op_jump_if_not_zero(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::JumpIfNotZero(value))
}

/// Push a jump loop start instruction into the byte-code stream.
///
/// Signature: ` -- `
fn word_jump_loop_start(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::JumpLoopStart)
}

/// Push a jump loop exit instruction into the byte-code stream.
///
/// Signature: ` -- `
fn word_jump_loop_exit(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::JumpLoopExit)
}

/// Push a jump target instruction into the byte-code stream.
///
/// Signature: `jump-label -- `
fn word_op_jump_target(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::JumpTarget(value))
}

/// Create a new block of byte-code instructions at the top of the generation stack.
///
/// Signature: ` -- `
fn word_code_new_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_mut().construction_new();
    Ok(())
}

/// Merge the top block of the byte-code generation stack into the one below it at the end of the
/// that block.
///
/// Signature: ` == `
fn word_code_merge_stack_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let code = interpreter.context_mut().construction_pop()?.code;

    interpreter.context_mut().construction_mut()?.code.extend(code);
    Ok(())
}

/// Pop the top block of the byte-code generation stack and push it onto the data stack.
///
/// Signature: ` -- code-block`
fn word_code_pop_stack_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let code = interpreter.context_mut().construction_pop()?.code;

    interpreter.push(code.to_value());
    Ok(())
}

/// Pop a code block from the top of the data stack and back onto the code generation stack.
///
/// Signature: `code-block -- `
fn word_code_push_stack_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let code = interpreter.pop_as_code()?;

    interpreter.context_mut().construction_new_with_code(code);
    Ok(())
}

/// Read the size in instructions of the code block at the top of the code generation stack.
///
/// Signature: ` -- block-size`
fn word_code_stack_block_size(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.context().construction()?.code.len().to_value();

    interpreter.push(value);
    Ok(())
}

/// Resolve all of the jump labels into relative addresses in the top code block.
///
/// Signature: ` -- `
fn word_code_resolve_jumps(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_mut().construction_mut()?.resolve_jumps();
    Ok(())
}

/// Compile incoming tokens in the token stream until one of the specified words is found.  The word
/// that was found is pushed onto the data stack.  Push the words to search for followed by the
/// count of words.  If none of the words are found, an error is generated.
///
/// Signature: `words .. word-count -- found-word`
fn word_code_compile_until_words(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    // Is the given token a word token and one of the words we're looking for?
    fn is_one_of_words(interpreter: &mut dyn Interpreter,
                       token: &Token,
                       words: &Vec<String>) -> Option<String>
    {
        if let Ok(found) = token.word(interpreter)
        {
            for word in words
            {
                if found == word
                {
                    return Some(found.clone());
                }
            }
        }

        None
    }

    // Build up the list of words to search for.
    let word_count = interpreter.pop_as_usize()?;
    let mut words = Vec::new();

    words.reserve(word_count);

    for _ in 0..word_count
    {
        words.push(interpreter.pop_as_string()?);
    }

    // Search for the words and error out if we hit the end of the token stream without finding
    // any of them.
    loop
    {
        // Get the next token if available.
        if let Ok(token) = interpreter.next_token()
        {
            // Is it a word we're looking for?
            if let Some(word) = is_one_of_words(interpreter, &token, &words)
            {
                interpreter.push(word.to_value());
                return Ok(());
            }
            else
            {
                // Nope, compile the token.
                process_token(interpreter, token)?;
            }
        }
        else
        {
            // We hit the end of the token stream without finding any of the words.  Generate a nice
            // error message that lists the words we were looking for.
            let mut message: String;

            if word_count == 1
            {
                message = format!("Could not find word {}.", words[0]);
            }
            else
            {
                message = "Could not find any of the words: ".to_string();

                for ( index, word ) in words.iter().enumerate()
                {
                    message.push_str(word);

                    if index < word_count - 1
                    {
                        message.push_str(", ");
                    }
                }

                message.push_str(".");
            }

            script_error(interpreter, message)?;
        }
    }
}

/// Set the insertion location for new instructions.  True means insert at the beginning of the
/// block, false means insert at the end.  Inserting at the end is the default.
///
/// Signature: `boolean -- `
fn word_code_insert_at_front(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let is_at_beginning = interpreter.pop_as_bool()?;

    interpreter.context_mut().insertion =
        if is_at_beginning
        {
            InsertionLocation::AtTop
        }
        else
        {
            InsertionLocation::AtEnd
        };

    Ok(())
}

/// Interpret and execute a string as if it were source code.
fn word_code_execute_source(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let source = interpreter.pop_as_string()?;
    interpreter.process_source(&"<repl>".to_string(), &source)
}



/// Register all of the byte-code generation words with the interpreter.
pub fn register_bytecode_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "op.def_variable", word_op_def_variable,
        "Insert this instruction into the byte stream.",
        "new-name -- ");

    add_native_word!(interpreter, "op.def_constant", word_op_def_constant,
        "Insert this instruction into the byte stream.",
        "new-name -- ");

    add_native_word!(interpreter, "op.read_variable", word_op_read_variable,
        "Insert this instruction into the byte stream.",
        " -- ");

    add_native_word!(interpreter, "op.write_variable", word_op_write_variable,
        "Insert this instruction into the byte stream.",
        " -- ");

    add_native_word!(interpreter, "op.execute", word_op_execute,
        "Insert this instruction into the byte stream.",
        "index -- ");

    add_native_word!(interpreter, "op.push_constant_value", word_op_push_constant_value,
        "Insert this instruction into the byte stream.",
        "value -- ");

    add_native_word!(interpreter, "op.mark_loop_exit", word_mark_loop_exit,
        "Insert this instruction into the byte stream.",
        "identifier -- ");

    add_native_word!(interpreter, "op.unmark_loop_exit", word_unmark_loop_exit,
        "Insert this instruction into the byte stream.",
        " -- ");

    add_native_word!(interpreter, "op.mark_catch", word_op_mark_catch,
        "Insert this instruction into the byte stream.",
        "identifier -- ");

    add_native_word!(interpreter, "op.unmark_catch", word_op_unmark_catch,
        "Insert this instruction into the byte stream.",
        " -- ");

    add_native_word!(interpreter, "op.jump", word_op_jump,
        "Insert this instruction into the byte stream.",
        "identifier -- ");

    add_native_word!(interpreter, "op.jump_if_zero", word_op_jump_if_zero,
        "Insert this instruction into the byte stream.",
        "identifier -- ");

    add_native_word!(interpreter, "op.jump_if_not_zero", word_op_jump_if_not_zero,
        "Insert this instruction into the byte stream.",
        "identifier -- ");

    add_native_word!(interpreter, "op.jump_loop_start", word_jump_loop_start,
        "Insert this instruction into the byte stream.",
        " -- ");

    add_native_word!(interpreter, "op.jump_loop_exit", word_jump_loop_exit,
        "Insert this instruction into the byte stream.",
        " -- ");

    add_native_word!(interpreter, "op.jump_target", word_op_jump_target,
        "Insert this instruction into the byte stream.",
        "identifier -- ");

    add_native_word!(interpreter, "code.new_block", word_code_new_block,
        "Create a new sub-block on the code generation stack.",
        " -- ");

    add_native_word!(interpreter, "code.merge_stack_block", word_code_merge_stack_block,
        "Merge the top code block into the one below.",
        " -- ");

    add_native_word!(interpreter, "code.pop_stack_block", word_code_pop_stack_block,
        "Pop a code block off of the code stack and onto the data stack.",
        " -- code_block");

    add_native_word!(interpreter, "code.push_stack_block", word_code_push_stack_block,
        "Pop a block from the data stack and back onto the code stack.",
        "code_block -- ");

    add_native_word!(interpreter, "code.stack_block_size@", word_code_stack_block_size,
        "Read the size of the code block at the top of the stack.",
        " -- code_size");

    add_native_word!(interpreter, "code.resolve_jumps", word_code_resolve_jumps,
        "Resolve all of the jumps in the top code block.",
        " -- ");

    add_native_word!(interpreter, "code.compile_until_words", word_code_compile_until_words,
        "Compile words until one of the given words is found.",
        "words... word_count -- found_word");

    add_native_word!(interpreter, "code.insert_at_front", word_code_insert_at_front,
        "When true new instructions are added beginning of the block.",
        "bool -- ");

    add_native_word!(interpreter, "code.execute_source", word_code_execute_source,
        "Interpret and execute a string like it is source code.",
        "string_to_execute -- ???");
}
