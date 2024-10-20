
use crate::{ lang::{ code::ByteCode,
                     compilation::CodeConstructor,
                     source_buffer::SourceLocation,
                     tokenizing::TokenList },
             runtime::{ data_structures::{ contextual_data::ContextualData,
                                           contextual_list::ContextualList,
                                           data_object::DataObjectPtr,
                                           dictionary::{ Dictionary, WordInfo },
                                           value::Value },
                         error } };



pub mod sorth_interpreter;
pub mod sub_interpreter;



#[derive(Clone)]
pub struct CallItem
{
    pub location: SourceLocation,
    pub word: String
}



pub type CallStack = Vec<CallItem>;

pub type VariableList = ContextualList<Value>;

pub type ValueStack = Vec<Value>;



pub trait InterpreterStack
{
    fn stack(&self) -> &ValueStack;

    fn push(&mut self, value: &Value);

    fn pop(&mut self) -> Value;
    fn pop_as_int(&mut self) -> i64;
    fn pop_as_float(&mut self) -> f64;
    fn pop_as_bool(&mut self) -> bool;
    fn pop_as_string(&mut self) -> String;
    fn pop_as_data_object(&mut self) -> DataObjectPtr;
}



pub trait CodeManagement
{
    fn context_new(&mut self, tokens: TokenList);
    fn context_drop(&mut self);

    fn context(&self) -> &CodeConstructor;
    fn context_mut(&mut self) -> &mut CodeConstructor;

    fn process_source_file(&mut self, path: &String) -> error::Result<()>;
    fn process_source(&mut self, path: &String, source: &String) -> error::Result<()>;

    fn execute_code(&mut self, name: &String, code: &ByteCode) -> error::Result<()>;
}



#[derive(Clone)]
pub struct WordHandlerInfo
{
    name: String,
    location: SourceLocation,
    handler: usize
}



pub trait WordManagement
{
    fn current_location(&self) -> &Option<SourceLocation>;

    fn add_word(&self);

    fn find_word(&self, word: &String) -> Option<WordInfo>;
    fn word_handler_info(&self, index: usize) -> Option<WordHandlerInfo>;
    fn inverse_name_list(&self) -> Vec<String>;

    fn execute_word(&mut self,
                    location: &Option<SourceLocation>,
                    word: &WordInfo) -> error::Result<()>;
    fn execute_word_named(&mut self,
                          location: &Option<SourceLocation>,
                          word: &String) -> error::Result<()>;
    fn execute_word_index(&mut self,
                          location: &Option<SourceLocation>,
                          index: usize) -> error::Result<()>;

    fn call_stack(&self) -> &CallStack;
}



pub struct SubThreadInfo
{
}



pub trait ThreadManagement
{
}



pub trait Interpreter : ContextualData +
                        InterpreterStack +
                        CodeManagement +
                        WordManagement +
                        ThreadManagement
{
    fn add_search_path(&mut self, path: &String) -> error::Result<()>;
    fn add_search_path_for_file(&mut self, file_path: &String) -> error::Result<()>;
    fn drop_search_path(&mut self);

    fn find_file(&self, path: & String) -> error::Result<String>;

    fn variables(&self) -> &VariableList;
    fn dictionary(&self) -> &Dictionary;
}
