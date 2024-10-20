
use crate::{ lang::{ code::{ ByteCode, CodeConstructor },
                     source_buffer::SourceLocation },
             runtime::{ data_structures::{ contextual_data::ContextualData,
                                           contextual_list::ContextualList,
                                           data_object::{ DataDefinitionList,
                                                          DataObject,
                                                          DataObjectPtr },
                                           dictionary::{ Dictionary, WordInfo },
                                           value::Value },
                        error,
                        interpreter::{ CallStack,
                                       CodeManagement,
                                       Interpreter,
                                       InterpreterStack,
                                       ThreadManagement,
                                       ValueStack,
                                       VariableList,
                                       WordHandlerInfo,
                                       WordManagement } } };



pub type SearchPaths = Vec<String>;

pub type WordList = Vec<WordHandlerInfo>;



pub struct SorthInterpreter
{
    search_paths: SearchPaths,

    stack: ValueStack,

    current_location: Option<SourceLocation>,
    call_stack: CallStack,

    data_definitions: DataDefinitionList,

    dictionary: Dictionary,
    word_handlers: WordList,

    variables: ContextualList<Value>
}


impl Interpreter for SorthInterpreter
{
    fn add_search_path(&mut self, path: &String)
    {
        self.search_paths.push(path.clone());
    }

    fn variables(&self) -> &VariableList
    {
        &self.variables
    }

    fn dictionary(&self) -> &Dictionary
    {
        &self.dictionary
    }
}


impl ContextualData for SorthInterpreter
{
    fn mark_context(&mut self)
    {
        self.dictionary.mark_context();
        self.data_definitions.mark_context();
        self.variables.mark_context();
    }

    fn release_context(&mut self)
    {
        self.dictionary.release_context();
        self.data_definitions.release_context();
        self.variables.release_context();
    }
}


impl InterpreterStack for SorthInterpreter
{
    fn stack(&self) -> &ValueStack
    {
        &self.stack
    }

    fn push(&mut self, _value: &Value)
    {
        //
    }

    fn pop(&mut self) -> Value
    {
        Value::default()
    }

    fn pop_as_int(&mut self) -> i64
    {
        0
    }

    fn pop_as_float(&mut self) -> f64
    {
        0.0
    }

    fn pop_as_bool(&mut self) -> bool
    {
        false
    }

    fn pop_as_string(&mut self) -> String
    {
        String::new()
    }

    fn pop_as_data_object(&mut self) -> DataObjectPtr
    {
        DataObject::new(&self.data_definitions[0])
    }
}


impl CodeManagement for SorthInterpreter
{
    fn code_constructor(&mut self) -> Option<&CodeConstructor>
    {
        None
    }

    fn process_source_file(_path: &String) -> error::Result<()>
    {
        Ok(())
    }

    fn process_source(_path: &String, _source: &String) -> error::Result<()>
    {
        Ok(())
    }

    fn execute_code(&mut self, _name: &String, _code: &ByteCode) -> error::Result<()>
    {
        Ok(())
    }
}


impl WordManagement for SorthInterpreter
{
    fn current_location(&self) -> &Option<SourceLocation>
    {
        &self.current_location
    }

    fn add_word(&self)
    {
    }

    fn find_word(&self, _word: &String) -> Option<&WordInfo>
    {
        None
    }

    fn word_handler_info(&self, _index: usize) -> Option<&WordHandlerInfo>
    {
        None
    }

    fn inverse_name_list(&self) -> Vec<String>
    {
        Vec::new()
    }

    fn execute_word(_location: &Option<SourceLocation>, _word: &WordInfo) -> error::Result<()>
    {
        Ok(())
    }

    fn execute_word_named(_location: &Option<SourceLocation>, _word: &String) -> error::Result<()>
    {
        Ok(())
    }

    fn execute_word_index(_location: &Option<SourceLocation>, _index: usize) -> error::Result<()>
    {
        Ok(())
    }

    fn call_stack(&self) -> &CallStack
    {
        &self.call_stack
    }
}


impl ThreadManagement for SorthInterpreter
{
}


impl SorthInterpreter
{
    pub fn new() -> SorthInterpreter
    {
        SorthInterpreter
            {
                search_paths: Vec::new(),

                stack: Vec::new(),

                current_location: None,
                call_stack: CallStack::new(),

                data_definitions: DataDefinitionList::new(),

                dictionary: Dictionary::new(),
                word_handlers: WordList::new(),

                variables: VariableList::new()
            }
    }
}
