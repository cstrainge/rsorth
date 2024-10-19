
use std::{ error::Error,
           fmt::{ self, Debug, Display, Formatter } };
use crate::{ runtime::interpreter::CallStack,
             lang::source_buffer::SourceLocation };



pub type Result<T> = std::result::Result<T, ScriptError>;



#[derive(Clone)]
pub struct ScriptError
{
    location: Option<SourceLocation>,
    error: String,
    call_stack: Option<CallStack>
}


impl Error for ScriptError
{
}


impl Display for ScriptError
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match &self.location
        {
            Some(location) => write!(f, "{}: {}", location, self.error),
            None => write!(f, "{}", self.error)
        }
    }
}


impl Debug for ScriptError
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{}", self)
    }
}


impl ScriptError
{
    pub fn new(location: Option<SourceLocation>,
               error: String,
               call_stack: Option<CallStack>) -> ScriptError
    {
        ScriptError
            {
                location,
                error,
                call_stack
            }
    }

    pub fn new_as_result<T>(location: Option<SourceLocation>,
                            error: String,
                            call_stack: Option<CallStack>) -> Result<T>
    {
        Err(ScriptError::new(location, error, call_stack))
    }

    pub fn location(&self) -> &Option<SourceLocation>
    {
        &self.location
    }

    pub fn error(&self) -> &String
    {
        &self.error
    }

    pub fn call_stack(&self) -> &Option<CallStack>
    {
        &self.call_stack
    }
}
