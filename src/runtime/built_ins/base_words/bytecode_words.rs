
use crate::{ add_native_word,
             lang::{ code::{ Instruction, Op },
                     compilation::{process_token, InsertionLocation},
                     tokenizing::Token },
             runtime::{ data_structures::value::ToValue,
                        error::{self, script_error},
                        interpreter::Interpreter } };



fn insert_user_instruction(interpreter: &mut dyn Interpreter, op: Op) -> error::Result<()>
{
    let instruction = Instruction::new(None, op);
    interpreter.context_mut().push_instruction(instruction)
}



fn word_op_def_variable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::DefVariable(value))
}

fn word_op_def_constant(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::DefConstant(value))
}

fn word_op_read_variable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::ReadVariable)

}

fn word_op_write_variable(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::WriteVariable)

}

fn word_op_execute(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::Execute(value))
}

fn word_op_push_constant_value(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::PushConstantValue(value))
}

fn word_mark_loop_exit(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::MarkLoopExit(value))

}

fn word_unmark_loop_exit(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::UnmarkLoopExit)
}

fn word_op_mark_catch(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::MarkCatch(value))
}

fn word_op_unmark_catch(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::UnmarkCatch)
}

fn word_op_jump(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::Jump(value))
}

fn word_op_jump_if_zero(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::JumpIfZero(value))
}

fn word_op_jump_if_not_zero(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::JumpIfNotZero(value))
}

fn word_jump_loop_start(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::JumpLoopStart)
}

fn word_jump_loop_exit(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    insert_user_instruction(interpreter, Op::JumpLoopExit)
}

fn word_op_jump_target(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.pop()?;
    insert_user_instruction(interpreter, Op::JumpTarget(value))
}

fn word_code_new_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_mut().construction_new();
    Ok(())
}

fn word_code_merge_stack_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let code = interpreter.context_mut().construction_pop()?.code;

    interpreter.context_mut().construction_mut()?.code.extend(code);
    Ok(())
}

fn word_code_pop_stack_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let code = interpreter.context_mut().construction_pop()?.code;

    interpreter.push(code.to_value());
    Ok(())
}

fn word_code_push_stack_block(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let code = interpreter.pop_as_code()?;

    interpreter.context_mut().construction_new_with_code(code);
    Ok(())
}

fn word_code_stack_block_size(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let value = interpreter.context().construction()?.code.len().to_value();

    interpreter.push(value);
    Ok(())
}

fn word_code_resolve_jumps(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_mut().construction_mut()?.resolve_jumps();
    Ok(())
}

fn word_code_compile_until_words(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
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

    let word_count = interpreter.pop_as_usize()?;
    let mut words = Vec::new();

    words.reserve(word_count);

    for _ in 0..word_count
    {
        words.push(interpreter.pop_as_string()?);
    }

    loop
    {
        if let Ok(token) = interpreter.next_token()
        {
            if let Some(word) = is_one_of_words(interpreter, &token, &words)
            {
                interpreter.push(word.to_value());
                return Ok(());
            }
            else
            {
                process_token(interpreter, token)?;
            }
        }
        else
        {
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

fn word_code_execute_source(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let source = interpreter.pop_as_string()?;
    interpreter.process_source(&"<repl>".to_string(), &source)
}



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
