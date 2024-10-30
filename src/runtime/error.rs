
use std::{ error::Error,
           fmt::{ self, Debug, Display, Formatter } };
use crate::{ runtime::interpreter::CallStack,
             lang::source_buffer::SourceLocation };

use super::interpreter::Interpreter;



pub type Result<T> = std::result::Result<T, ScriptError>;



/// Any error that occurs during the execution of a Strange Forth script.
#[derive(Clone)]
pub struct ScriptError
{
    /// The location in the source code the error occurred, if available.
    location: Option<SourceLocation>,

    /// The description of the error.
    error: String,

    /// The script's call stack at the time of the error, if available.
    call_stack: Option<CallStack>
}


impl Error for ScriptError
{
}


/// Pretty print the ScriptError for debugging the error that occurred within the Strange Forth
/// script.
impl Display for ScriptError
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match &self.location
        {
            Some(location) => write!(f, "{}: {}", location, self.error)?,
            None => write!(f, "{}", self.error)?
        }

        if let Some(call_stack) = &self.call_stack
        {
            write!(f, "\n\nCall stack\n")?;

            for item in call_stack.iter().rev()
            {
                write!(f, "  {}\n", item)?;
            }
        }

        Ok(())
    }
}


/// Pretty print the ScriptError for debugging the error that occurred within the Strange Forth
/// script.
impl Debug for ScriptError
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{}", self)
    }
}


impl ScriptError
{
    /// Create a new ScriptError.
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

    /// Create a new Script Error and wrap it in a Result::Err.
    pub fn new_as_result<T>(location: Option<SourceLocation>,
                            error: String,
                            call_stack: Option<CallStack>) -> Result<T>
    {
        Err(ScriptError::new(location, error, call_stack))
    }

    /// If available, the location in the source code the error occurred.
    pub fn location(&self) -> &Option<SourceLocation>
    {
        &self.location
    }

    /// The description of the error.
    pub fn error(&self) -> &String
    {
        &self.error
    }

    /// If available, the script's call stack at the time of the error.
    pub fn call_stack(&self) -> &Option<CallStack>
    {
        &self.call_stack
    }
}


/// Allow for the conversion of a std::io::Error into a ScriptError.
impl From<std::io::Error> for ScriptError
{
    fn from(error: std::io::Error) -> ScriptError
    {
        ScriptError::new(None, format!("I/O error: {}", error), None)
    }
}



/// A convenience function for creating a ScriptError and wrapping in in a Result::Err using the
/// interpreter's current location and call stack.
pub fn script_error<T>(interpreter: &dyn Interpreter, message: String) -> Result<T>
{
    let location = interpreter.current_location().clone();
    let call_stack = interpreter.call_stack().clone();

    ScriptError::new_as_result(location, message, Some(call_stack))
}



pub fn script_error_str<T>(interpreter: &dyn Interpreter, message: &str) -> Result<T>
{
    script_error(interpreter, message.to_string())
}
