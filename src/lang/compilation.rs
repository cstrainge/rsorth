
use std::collections::HashMap;
use crate::{ lang::{ code::{ ByteCode, Instruction, Op },
                     source_buffer::SourceLocation,
                     tokenizing::{ Token, TokenList } },
             runtime::{ data_structures::{ dictionary::{ WordRuntime,
                                                         WordVisibility },
                                           value::{ ToValue, Value } },
                        error::{ self, ScriptError },
                        interpreter::Interpreter } };



#[derive(Clone)]
pub struct Construction
{
    pub runtime: WordRuntime,
    pub visibility: WordVisibility,

    pub name: String,
    pub location: SourceLocation,
    pub description: String,
    pub signature: String,

    pub code: ByteCode
}


impl Construction
{
    pub fn new() -> Construction
    {
        Construction
            {
                runtime: WordRuntime::Normal,
                visibility: WordVisibility::Visible,

                name: String::new(),
                location: SourceLocation::new(),
                description: String::new(),
                signature: String::new(),

                code: ByteCode::new()
            }
    }

    pub fn resolve_jumps(&mut self)
    {
        fn is_jump(instruction: &Instruction) -> bool
        {
            match instruction.op
            {
                Op::Jump(_)          |
                Op::JumpIfZero(_)    |
                Op::JumpIfNotZero(_) |
                Op::MarkLoopExit(_)  |
                Op::MarkCatch(_)       => true,
                _                      => false
            }
        }

        fn jump_label(instruction: &Instruction) -> Option<String>
        {
            match &instruction.op
            {
                Op::Jump(value)          => value.get_string_val(),
                Op::JumpIfZero(value)    => value.get_string_val(),
                Op::JumpIfNotZero(value) => value.get_string_val(),
                Op::MarkLoopExit(value)  => value.get_string_val(),
                Op::MarkCatch(value)     => value.get_string_val(),
                _                        => None
            }
        }

        fn update_jump_op(jump_op: &Op, relative: i64) -> Op
        {
            match jump_op
            {
                Op::Jump(_)          => Op::Jump(relative.to_value()),
                Op::JumpIfZero(_)    => Op::JumpIfZero(relative.to_value()),
                Op::JumpIfNotZero(_) => Op::JumpIfNotZero(relative.to_value()),
                Op::MarkLoopExit(_)  => Op::MarkLoopExit(relative.to_value()),
                Op::MarkCatch(_)     => Op::MarkCatch(relative.to_value()),
                _                    => panic!("Invalid jump operation!")
            }
        }

        let mut jump_indices = Vec::<i64>::new();
        let mut jump_targets = HashMap::<String, i64>::new();

        for index in 0..self.code.len()
        {
            if is_jump(&self.code[index])
            {
                jump_indices.push(index as i64);
            }
            else if let Op::JumpTarget(value) = &self.code[index].op
            {
                jump_targets.insert(value.to_string(), index as i64);
                self.code[index].op = Op::JumpTarget(Value::None);
            }
        }

        for jump_index in jump_indices
        {
            if let Some(jump_label) = jump_label(&self.code[jump_index as usize])
            {
                let target_index = jump_targets[&jump_label];
                let relative = target_index - jump_index;
                let jump_op = &self.code[jump_index as usize].op;

                self.code[jump_index as usize].op = update_jump_op(&jump_op, relative);
            }
        }
    }
}



pub type ConstructionList = Vec<Construction>;



pub enum InsertionLocation
{
    AtEnd,
    AtTop
}



pub struct CodeConstructor
{
    pub constructions: ConstructionList,
    pub insertion: InsertionLocation,
    pub input: TokenList,
    pub current: usize
}



impl CodeConstructor
{
    pub fn new(token_list: TokenList) -> CodeConstructor
    {
        CodeConstructor
            {
                constructions: vec![ Construction::new() ],
                insertion: InsertionLocation::AtEnd,
                input: token_list,
                current: 0
            }
    }

    pub fn next_token(&mut self) -> Option<Token>
    {
        if self.current >= self.input.len()
        {
            return None;
        }

        let token = &self.input[self.current];
        self.current += 1;

        Some(token.clone())
    }

    pub fn construction_new(&mut self)
    {
        self.constructions.push(Construction::new());
    }

    pub fn construction_new_with_code(&mut self, code: ByteCode)
    {
        let mut construction = Construction::new();

        construction.code = code;
        self.constructions.push(construction);
    }

    pub fn construction_pop(&mut self) -> error::Result<Construction>
    {
        if self.constructions.len() == 0
        {
            ScriptError::new_as_result(None,
                                       "No construction to pop.".to_string(),
                                       None)?;
        }

        Ok(self.constructions.pop().unwrap())
    }

    pub fn construction(&self) -> error::Result<&Construction>
    {
        if self.constructions.is_empty()
        {
            ScriptError::new_as_result(None,
                                       "Accessing an empty construction context.".to_string(),
                                       None)?;
        }

        let index = self.constructions.len() - 1;
        Ok(&self.constructions[index])
    }

    pub fn construction_mut(&mut self) -> error::Result<&mut Construction>
    {
        if self.constructions.is_empty()
        {
            ScriptError::new_as_result(None,
                                       "Accessing an empty construction context.".to_string(),
                                       None)?;
        }

        let index = self.constructions.len() - 1;
        Ok(&mut self.constructions[index])
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> error::Result<()>
    {
        if let InsertionLocation::AtEnd = self.insertion
        {
            self.construction_mut()?.code.push(instruction);
        }
        else
        {
            self.construction_mut()?.code.insert(0, instruction);
        }

        Ok(())
    }
}


pub type CodeConstructorList = Vec<CodeConstructor>;



pub fn process_token(interpreter: &mut dyn Interpreter,
                     token: Token)-> error::Result<()>
{
    fn token_to_word_name(token: &Token) -> Option<( SourceLocation, String )>
    {
        match token
        {
            Token::Word(location, name)     => Some(( location.clone(), name.clone() )),
            Token::Number(location, number) => Some(( location.clone(), number.to_string() )),
            Token::String(_, _)             => None
        }
    }

    if    let Some(( location, name )) = token_to_word_name(&token)
       && let Some(word_info) = interpreter.find_word(&name)
    {
        if let WordRuntime::Immediate = word_info.runtime
        {
            interpreter.execute_word(&Some(location), &word_info.clone())?;
        }
        else
        {
            let index = word_info.handler_index as i64;
            let instruction = Instruction::new(Some(location), Op::Execute(index.to_value()));

            interpreter.context_mut().push_instruction(instruction)?;
        }
    }
    else
    {
        match token
        {
            Token::Word(location, name) =>
                {
                    let instruction = Instruction::new(Some(location),
                                                       Op::Execute(name.to_value()));

                    interpreter.context_mut().push_instruction(instruction)?;
                },

            Token::Number(location, number) =>
                {
                    let instruction = Instruction::new(Some(location),
                                                       Op::PushConstantValue(number.to_value()));

                    interpreter.context_mut().push_instruction(instruction)?;
                },

            Token::String(location, text) =>
                {
                    let instruction = Instruction::new(Some(location),
                                                       Op::PushConstantValue(text.to_value()));

                    interpreter.context_mut().push_instruction(instruction)?;
                }
        }
    }

    Ok(())
}



pub fn process_source_from_tokens(tokens: TokenList,
                                  interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_new(tokens);

    while let Some(token) = interpreter.context_mut().next_token()
    {
        if let Err(error) = process_token(interpreter, token)
        {
            interpreter.context_drop()?;
            return Err(error);
        }
    }

    let code =
        {
            let construction = interpreter.context().construction();

            if let Err(error) = construction
            {
                interpreter.context_drop()?;
                return Err(error);
            }

            let code = construction.unwrap().code.clone();
            interpreter.context_drop()?;

            code
        };

    interpreter.execute_code(&"<toplevel>".to_string(), &code)
}
