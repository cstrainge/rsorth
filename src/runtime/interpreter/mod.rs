
use std::{ fmt::{ self, Display, Formatter }, rc::Rc };
use crate::{ lang::{ code::ByteCode,
                     compilation::CodeConstructor,
                     source_buffer::SourceLocation,
                     tokenizing::TokenList },
             runtime::{ data_structures::{ contextual_data::ContextualData,
                                           contextual_list::ContextualList,
                                           data_object::DataObjectPtr,
                                           dictionary::{ Dictionary,
                                                         WordInfo,
                                                         WordRuntime,
                                                         WordType,
                                                         WordVisibility },
                                           value::Value },
                         error } };



pub mod sorth_interpreter;
pub mod sub_interpreter;



#[derive(Clone)]
pub struct CallItem
{
    location: SourceLocation,
    word: String
}


impl CallItem
{
    pub fn new(word: String, location: SourceLocation) -> CallItem
    {
        CallItem
            {
                location,
                word
            }
    }

    pub fn location(&self) -> &SourceLocation
    {
        &self.location
    }

    pub fn word(&self) -> &String
    {
        &self.word
    }
}


impl Display for CallItem
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{}: {}", self.location, self.word)
    }
}



pub type CallStack = Vec<CallItem>;

pub type VariableList = ContextualList<Value>;

pub type ValueStack = Vec<Value>;



pub trait InterpreterStack
{
    fn stack(&self) -> &ValueStack;

    fn push(&mut self, value: &Value);

    fn pop(&mut self) -> error::Result<Value>;
    fn pop_as_int(&mut self) -> error::Result<i64>;
    fn pop_as_float(&mut self) -> error::Result<f64>;
    fn pop_as_bool(&mut self) -> error::Result<bool>;
    fn pop_as_string(&mut self) -> error::Result<String>;
    fn pop_as_data_object(&mut self) -> error::Result<DataObjectPtr>;
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



pub type WordHandler = dyn Fn(&mut dyn Interpreter) -> error::Result<()>;



#[derive(Clone)]
pub struct WordHandlerInfo
{
    name: String,
    location: SourceLocation,
    handler: Rc<WordHandler>
}


impl WordHandlerInfo
{
    pub fn new(name: String, location: SourceLocation, handler: Rc<WordHandler>) -> WordHandlerInfo
    {
        WordHandlerInfo
            {
                name,
                location,
                handler
            }
    }

    pub fn name(&self) -> &String
    {
        &self.name
    }

    pub fn location(&self) -> &SourceLocation
    {
        &self.location
    }

    pub fn handler(&self) -> Rc<WordHandler>
    {
        self.handler.clone()
    }
}



#[macro_export]
macro_rules! add_native_word
{
    ( $interpreter:expr ,
      $name:expr ,
      $function:expr ,
      $description:expr ,
      $signature:expr) =>
    {
        {
            use std::rc::Rc;
            use crate::runtime::data_structures::dictionary::{ WordRuntime,
                                                               WordVisibility,
                                                               WordType };

            $interpreter.add_word(file!().to_string(),
                                  line!() as usize,
                                  column!() as usize,
                                  $name.to_string(),
                                  Rc::new($function),
                                  $description.to_string(),
                                  $signature.to_string(),
                                  WordRuntime::Normal,
                                  WordVisibility::Visible,
                                  WordType::Native);
        }
    };
}



#[macro_export]
macro_rules! add_native_immediate_word
{
    ( $interpreter:expr ,
      $name:literal ,
      $function:expr ,
      $description:literal ,
      $signature:literal) =>
    {
        {
            use std::rc::Rc;
            use crate::runtime::data_structures::dictionary::{ WordRuntime,
                                                               WordVisibility,
                                                               WordType };

            interpreter.add_word(file!().to_string(),
                                 line!() as usize,
                                 column!() as usize,
                                 $name.to_string(),
                                 Rc::new($function),
                                 $description.to_string(),
                                 $signature.to_string(),
                                 WordRuntime::Immediate,
                                 WordVisibility::Visible,
                                 WordType::Native);
        }
    };
}



pub trait WordManagement
{
    fn current_location(&self) -> &Option<SourceLocation>;

    fn add_word(&mut self,
                file: String,
                line: usize,
                column: usize,
                name: String,
                handler: Rc<WordHandler>,
                description: String,
                signature: String,
                runtime: WordRuntime,
                visibility: WordVisibility,
                word_type: WordType);

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

    fn call_stack_push(&mut self, name: String, location: SourceLocation);
    fn call_stack_pop(&mut self) -> error::Result<()>;
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
