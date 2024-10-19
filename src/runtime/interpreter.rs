
use crate::{ lang::source_buffer::SourceLocation,
             runtime::data_structures::{ contextual_data::ContextualData,
                                         contextual_list::ContextualList,
                                         dictionary::Dictionary,
                                         value::Value } };



pub type SearchPaths = Vec<String>;



#[derive(Clone)]
pub struct CallItem
{
    pub location: SourceLocation,
    pub word: String
}



pub type CallStack = Vec<CallItem>;



pub struct WordHandlerInfo
{
}



pub type WordList = Vec<WordHandlerInfo>;



pub struct SorthInterpreter
{
    search_paths: SearchPaths,

    stack: Vec<Value>,

    current_location: Option<SourceLocation>,
    call_stack: CallStack,

    dictionary: Dictionary,
    word_handlers: WordList,

    variables: ContextualList<Value>
}


impl ContextualData for SorthInterpreter
{
    fn mark_context(&mut self)
    {
        self.dictionary.mark_context();
        self.variables.mark_context();
    }

    fn release_context(&mut self)
    {
        self.dictionary.release_context();
        self.variables.release_context();
    }
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

                dictionary: Dictionary::new(),
                word_handlers: WordList::new(),

                variables: ContextualList::new()
            }
    }
}
