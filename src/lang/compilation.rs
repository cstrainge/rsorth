
use crate::{ lang::{ code::{ ByteCode, Instruction, Op },
                     source_buffer::SourceLocation,
                     tokenizing::{ tokenize_from_file, tokenize_from_source, Token, TokenList } },
             runtime::{ data_structures::{ dictionary::{ WordRuntime,
                                                         WordVisibility },
                                           value::ToValue },
                        error,
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
                constructions: ConstructionList::new(),
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

    pub fn construction(&self) -> &Construction
    {
        let index = self.constructions.len() - 1;
        &self.constructions[index]
    }

    pub fn construction_mut(&mut self) -> &mut Construction
    {
        let index = self.constructions.len() - 1;
        &mut self.constructions[index]
    }

    pub fn push_instruction(&mut self, instruction: Instruction)
    {
        self.construction_mut().code.push(instruction);
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
            interpreter.execute_word(&Some(location), &word_info)?;
        }
        else
        {
            let index = word_info.handler_index as i64;
            let instruction = Instruction::new(Some(location), Op::Execute(index.to_value()));

            interpreter.context_mut().push_instruction(instruction);
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

                    interpreter.context_mut().push_instruction(instruction);
                },

            Token::Number(location, number) =>
                {
                    let instruction = Instruction::new(Some(location),
                                                       Op::PushConstantValue(number.to_value()));

                    interpreter.context_mut().push_instruction(instruction);
                },

            Token::String(location, text) =>
                {
                    let instruction = Instruction::new(Some(location),
                                                       Op::PushConstantValue(text.to_value()));

                    interpreter.context_mut().push_instruction(instruction);
                }
        }
    }

    Ok(())
}



fn process_source_from_tokens(path: &String,
                              tokens: TokenList,
                              interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_new(tokens);

    while let Some(token) = interpreter.context_mut().next_token()
    {
        process_token(interpreter, token)?;
    }

    let code = interpreter.context().construction().code.clone();
    interpreter.context_drop();

    interpreter.execute_code(path, &code)
}


pub fn process_source_from_string(path: &String,
                                  source_text: &String,
                                  interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let tokens = tokenize_from_source(path, source_text)?;
    process_source_from_tokens(path, tokens, interpreter)
}


pub fn process_source_from_file(path: &String,
                                interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let full_path = interpreter.find_file(path)?;
    let tokens = tokenize_from_file(path)?;

    interpreter.add_search_path(&full_path)?;

    let result = process_source_from_tokens(path, tokens, interpreter);

    interpreter.drop_search_path();

    result
}
