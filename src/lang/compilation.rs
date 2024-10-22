
use crate::{ lang::{ code::{ ByteCode, Instruction, Op },
                     source_buffer::SourceLocation,
                     tokenizing::{ Token, TokenList } },
             runtime::{ data_structures::{ dictionary::{ WordRuntime,
                                                         WordVisibility },
                                           value::ToValue },
                        error::{self, ScriptError},
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
        self.construction_mut()?.code.push(instruction);
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
