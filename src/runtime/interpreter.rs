
use crate::runtime::data_structures::{ contextual_data::ContextualData,
                                       contextual_list::ContextualList,
                                       dictionary::Dictionary,
                                       value::Value };



pub struct SorthInterpreter
{
    stack: Vec<Value>,

    dictionary: Dictionary,

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
                stack: Vec::new(),
                dictionary: Dictionary::new(),
                variables: ContextualList::new()
            }
    }
}
