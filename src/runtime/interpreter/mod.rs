
use std::{ fmt::{ self,
                  Display,
                  Formatter },
           rc::Rc };
use crate::{ lang::{ code::{ ByteCode,
                             Instruction,
                             Op },
                     compilation::CodeConstructor,
                     source_buffer::SourceLocation,
                     tokenizing::{ NumberType,
                                   Token,
                                   TokenList } },
             runtime::{ data_structures::{ byte_buffer::ByteBufferPtr,
                                           contextual_data::ContextualData,
                                           contextual_list::ContextualList,
                                           data_object::{ DataDefinitionList,
                                                          DataObjectDefinitionPtr,
                                                          DataObjectPtr },
                                           dictionary::{ Dictionary,
                                                         WordInfo,
                                                         WordRuntime,
                                                         WordType,
                                                         WordVisibility },
                                           value::Value,
                                           value_hash::ValueHashPtr,
                                           value_vec::ValueVecPtr },
                         error } };



pub mod sorth_interpreter;
pub mod sub_interpreter;



/// A call stack item is a record of the executing word's name ad the location within the original
/// source code from which it was found.  This items are read-only and the fields are accessed by
/// member functions.
#[derive(Clone)]
pub struct CallItem
{
    location: SourceLocation,
    word: String
}


impl CallItem
{
    /// Create a new call stack item.
    pub fn new(word: String, location: SourceLocation) -> CallItem
    {
        CallItem
            {
                location,
                word
            }
    }


    /// Where in the source code was the execution of this word found?
    pub fn location(&self) -> &SourceLocation
    {
        &self.location
    }


    // The name of the word being executed.
    pub fn word(&self) -> &String
    {
        &self.word
    }
}


/// Make sure that this word can be nicely displayed to the user in event of an error.
impl Display for CallItem
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{}: {}", self.location, self.word)
    }
}



/// Type to represent a call stack.  This is a stack of call items currently being executed by the
/// interpreter.  This is used to help track errors and provide a scripts stack trace to the user.
pub type CallStack = Vec<CallItem>;

/// Type to represent a list of variables managed by the interpreter.  This is a list of values that
/// keep track of the current context.  If a context is released all variables within that context
/// are also lost.
pub type VariableList = ContextualList<Value>;

/// The data stack of values managed by the interpreter.
pub type ValueStack = Vec<Value>;



/// Trait for managing the interpreter's data stack.  Intended to be called by immediate/normal
/// words, both native and scripted.
pub trait InterpreterStack
{
    /// What has the maximum depth the stack as reached so far?
    fn stack_max_depth(&self) -> usize;


    /// Use to examine the full data stack when required.  One example is for the stack dump command
    /// `.s` in the repl.  This can only fail if we run out of memory for allocating a spot on the
    /// data stack.
    fn stack(&self) -> &ValueStack;


    /// Push a script value onto the stack.  This is the primary way of sending values to words.
    /// Only values supported by the Value enumeration are supported on the data stack.
    fn push(&mut self, value: Value);


    /// Pop a value from the stack.  This is the primary way of receiving outputs from words.  Only
    /// values supported by the Value enumeration are supported on the data stack.  If the stack is
    /// empty a stack underflow error is returned.
    fn pop(&mut self) -> error::Result<Value>;

    /// Pop the top value and attempt to convert it to an integer.  If the value can not be
    /// converted an error is returned.  We also fail if the stack is empty.
    fn pop_as_int(&mut self) -> error::Result<i64>;

    /// Pop the top value and attempt to convert it to a usize.  If the value can not be converted
    /// an error is returned.  We also fail if the stack is empty.
    fn pop_as_usize(&mut self) -> error::Result<usize>;

    /// Pop the top value and attempt to convert it to a floating point value.  If the value can not
    /// be converted an error is returned.  We also fail if the stack is empty.
    fn pop_as_float(&mut self) -> error::Result<f64>;

    /// Pop the top value and attempt to convert it to a boolean.  If the value can not be converted
    /// an error is returned.  We also fail if the stack is empty.
    fn pop_as_bool(&mut self) -> error::Result<bool>;

    /// Pop the top value and attempt to convert it to a string.  If the value can not be converted
    /// an error is returned.  We also fail if the stack is empty.
    fn pop_as_string(&mut self) -> error::Result<String>;

    /// Pop the top value and attempt to convert it to an array of values.  If the value can not be
    /// converted an error is returned.  We also fail if the stack is empty.
    fn pop_as_array(&mut self) -> error::Result<ValueVecPtr>;

    /// Pop the top value and attempt to convert it to a hash map.  If the value can not be
    /// converted an error is returned.  We also fail if the stack is empty.
    fn pop_as_hash_map(&mut self) -> error::Result<ValueHashPtr>;

    /// Pop the top value and attempt to convert it to a a data object, or aka a Forth struct.  If
    /// the value can not be converted an error is returned.  We also fail if the stack is empty.
    fn pop_as_data_object(&mut self) -> error::Result<DataObjectPtr>;

    /// Pop the top value and attempt to convert it to a byte buffer.  If the value can not be
    /// converted an error is returned.  We also fail if the stack is empty.
    fn pop_as_byte_buffer(&mut self) -> error::Result<ByteBufferPtr>;

    /// Pop the top value and attempt to convert it to a code block.  If the value can not be
    /// converted an error is returned.  We also fail if the stack is empty.
    fn pop_as_code(&mut self) -> error::Result<ByteCode>;


    /// Pick a value from the given index in the stack and return it, shrinking the stack by one.
    /// If the index is out of bounds a stack underflow error is returned.
    fn pick(&mut self, index: usize) -> error::Result<Value>;

    /// Push a value into the specified index within the stack.  If the index is to the end of the
    /// stack the value is added to the bottom of the stack.
    fn push_to(&mut self, index: usize) -> error::Result<()>;
}



/// Trait for managing, compiling, and executing bytecode as well as managing the incoming source
/// code token stream.
///
/// These functions are only properly available during a script's "compile-time."  Thus they should
/// only be called from immediate words, either native or scripted.
pub trait CodeManagement
{
    /// Get the next token from the current source code's token stream.
    fn next_token(&mut self) -> error::Result<Token>;

    /// Get the next token as text from the current source code's token stream.  This will succeed
    /// if the next token is either a word or a string.
    fn next_token_text(&mut self) -> error::Result<String>;

    /// Get the next token as a string from the current source code's token stream.  This only
    /// succeeds if the next token represents a string in the source code.
    fn next_token_string(&mut self) -> error::Result<String>;

    /// Get the next token as a number from the current source code's token stream.  This only
    /// succeeds if the next token represents a number in the original source code.  The number can
    /// either be a 64-bit integer or floating point number.
    fn next_token_number(&mut self) -> error::Result<NumberType>;

    /// Get the next token as a word from the current source code's token stream.  This will only
    /// succeed if the next token represents a word in the original source code.
    fn next_token_word(&mut self) -> error::Result<( SourceLocation, String )>;


    /// Insert a byte-code instruction into the current context's instruction stream.  The stream it
    /// self is stack based, so only the top byte-code stream is updated.
    fn insert_user_instruction(&mut self,
                               location: Option<SourceLocation>,
                               op: Op) -> error::Result<()>
    {
        let instruction = Instruction::new(location, op);
        self.context_mut().push_instruction(instruction)
    }


    /// Create a new compilation context for a given source code token list.  This context is used
    /// to compile the source code into byte-code.
    fn context_new(&mut self, tokens: TokenList);

    // Drop the top compilation context from the stack as it's no longer needed.
    fn context_drop(&mut self) -> error::Result<()>;


    /// Access the current compilation context and it's byte-code stream.
    fn context(&self) -> &CodeConstructor;

    /// Access the current compilation context as mutable and it's byte-code stream.
    fn context_mut(&mut self) -> &mut CodeConstructor;


    /// Compile a Forth script from a source file.  This will read the file, tokenize it and compile
    /// it into byte-code.  All immediate words defined within and without will be executed in order
    /// to help process the source code.
    fn process_source_file(&mut self, path: &String) -> error::Result<()>;

    /// Compile a Forth script from an in memory source string.  This will tokenize it and compile
    /// it into byte-code.  All immediate words defined within and without will be executed in order
    /// to help process the source code.
    ///
    /// The path parameter is used to represent the source code in things like call stacks and error
    /// reporting.  For example, the repl uses a path of "\<repl\>" to represent source code entered
    /// by the user.
    fn process_source(&mut self, path: &String, source: &String) -> error::Result<()>;


    /// Execute a bytecode block and associate a name with that code for use in error reporting.
    fn execute_code(&mut self, name: &String, code: &ByteCode) -> error::Result<()>;
}



/// Definition of a word handler function.  This is the function that is called when a word is to be
/// executed.  Can be a lambda, a callable object or a Rust function.
pub type WordHandler = dyn Fn(&mut dyn Interpreter) -> error::Result<()>;



/// Information about a word handler.  Once created it's fields are read-only and accessed by member
/// methods.
#[derive(Clone)]
pub struct WordHandlerInfo
{
    name: String,
    location: SourceLocation,
    handler: Rc<WordHandler>
}


/// Core implementation of WordHandlerInfo's methods.
impl WordHandlerInfo
{
    /// Create a new WordHandlerInfo instance.
    pub fn new(name: String, location: SourceLocation, handler: Rc<WordHandler>) -> WordHandlerInfo
    {
        WordHandlerInfo
            {
                name,
                location,
                handler
            }
    }

    /// The name of the word itself.
    pub fn name(&self) -> &String
    {
        &self.name
    }

    /// Where this word was defined in the original source code.  Can be from either in Forth or
    /// Rust source code.
    pub fn location(&self) -> &SourceLocation
    {
        &self.location
    }

    /// The Handler function for the word.  It can be a native or a scripted word.
    pub fn handler(&self) -> Rc<WordHandler>
    {
        self.handler.clone()
    }
}



/// Simplify registering a native regular word with the interpreter.
///
/// Required parameters are, the interpreter instance to register with.  The name of the word to
/// register.  The word function handler to execute for the word.  A simple description of the word.
/// As well as the word's stack signature.
#[macro_export]
macro_rules! add_native_word
{
    (
        $interpreter:expr ,
        $name:expr ,
        $function:expr ,
        $description:expr ,
        $signature:expr
    ) =>
    {
        {
            // Import the necessary items for the macro to work.
            use std::rc::Rc;
            use crate::runtime::data_structures::dictionary::{ WordRuntime,
                                                               WordVisibility,
                                                               WordType };

            // Register the word while recording where in the source code the word was registered
            // from.
            $interpreter.add_word(file!().to_string(),       // Original source location that this
                                  line!() as usize,          //  word was registered from.
                                  column!() as usize,
                                  $name.to_string(),         // Name.
                                  Rc::new($function),        // Function handler.
                                  $description.to_string(),  // Word description.
                                  $signature.to_string(),    // Word signature.
                                  WordRuntime::Normal,       // The word runs at run time.
                                  WordVisibility::Visible,   // The word is visible in the index.
                                  WordType::Native);         // This is a native word.
        }
    };
}



/// Simplify registering a native immediate word with the interpreter.  That is, this word is
/// intended to be executed at compile time.
///
/// Required parameters are, the interpreter instance to register with.  The name of the word to
/// register.  The word function handler to execute for the word.  A simple description of the word.
/// As well as the word's stack signature.
#[macro_export]
macro_rules! add_native_immediate_word
{
    (
        $interpreter:expr ,
        $name:literal ,
        $function:expr ,
        $description:literal ,
        $signature:literal
    ) =>
    {
        {
            // Import the necessary items for the macro to work.
            use std::rc::Rc;
            use crate::runtime::data_structures::dictionary::{ WordRuntime,
                                                               WordVisibility,
                                                               WordType };

            // Register the word while recording where in the source code the word was registered
            // from.
            $interpreter.add_word(file!().to_string(),       // Original source location that this
                                  line!() as usize,          //  word was registered from.
                                  column!() as usize,
                                  $name.to_string(),         // Name.
                                  Rc::new($function),        // Function handler.
                                  $description.to_string(),  // Word description.
                                  $signature.to_string(),    // Word signature.
                                  WordRuntime::Immediate,    // The word runs at compile time.
                                  WordVisibility::Visible,   // The word is visible in the index.
                                  WordType::Native);         // This is a native word.
        }
    };
}



/// Trait for managing and executing words known to the interpreter.
pub trait WordManagement
{
    /// If currently set, this represents the current executing location in the original Forth
    /// source code.
    fn current_location(&self) -> &Option<SourceLocation>;


    /// Add a new word to the interpreter's dictionary.  This can be a native word or a scripted
    /// word.
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


    /// Add a new structure definition to the definition list.
    fn add_structure_definition(&mut self, definition_ptr: DataObjectDefinitionPtr);


    //// Find a word in the interpreter's dictionary by name.
    fn find_word(&self, word: &String) -> Option<&WordInfo>;

    /// Get a word's execution information from it's handler index.
    fn word_handler_info(&self, index: usize) -> Option<&WordHandlerInfo>;

    /// Get a lookup list of word names indexed by their handler index.
    fn inverse_name_list(&self) -> Vec<String>;


    /// Execute a word handler by it's handler information.
    fn execute_word_handler(&mut self,
                            location: &SourceLocation,
                            word_handler_info: &WordHandlerInfo) -> error::Result<()>;

    /// Find and execute a word by WordInfo.  Supply a source location to represent where the word
    /// was executed from.  Use the macro `location_here!()` to get the current location in the Rust
    /// source code if the word is executed from native code.
    ///
    /// If the word is not found an error is returned.  Otherwise the word is executed it's result
    /// is returned.
    fn execute_word(&mut self,
                    location: &SourceLocation,
                    word: &WordInfo) -> error::Result<()>;

    /// Find and execute a word by name.  Supply a source location to represent where the word was
    /// executed from.  Use the macro `location_here!()` to get the current location in the Rust
    /// source code if the word is executed from native code.
    ///
    /// If the word is not found a script error is returned.  Otherwise the word is executed and
    /// it's result is returned.
    fn execute_word_named(&mut self,
                          location: &SourceLocation,
                          word: &String) -> error::Result<()>;

    /// Execute a word by it's handler index.  Supply a source location to represent where the word
    /// was executed from.  Use the macro `location_here!()` to get the current location in the Rust
    /// source code if the word is executed from native code.
    ///
    /// If the word index exceeds the bounds of the word list, a script error is returned.
    /// Otherwise the word is executed and it's result is returned.
    fn execute_word_index(&mut self,
                          location: &SourceLocation,
                          index: usize) -> error::Result<()>;


    /// The current script execution call stack.
    fn call_stack(&self) -> &CallStack;


    /// Push a new name and location onto the call stack.  This information is used to help track
    /// errors reported by the interpreter.
    fn call_stack_push(&mut self, name: String, location: SourceLocation);

    /// Pop the last name and location from the call stack.
    fn call_stack_pop(&mut self) -> error::Result<()>;
}



/// To be implemented...
/*pub struct SubThreadInfo
{
}*/


/// Interpreter thread management trait.
///
/// Define the functionality for managing the threads in the Strange Forth interpreter.
pub trait ThreadManagement
{
}



/// Core interpreter trait.
///
/// This trait defines and brings together the traits that define the core functionality of the
/// Strange Forth interpreter.
///
/// Functionality includes, marking and releasing of contexts.  Managing the Forth data stack.
/// Managing and executing bytecode and words.  As well as managing interpreter sub-threads for user
/// code.
pub trait Interpreter : ContextualData +
                        InterpreterStack +
                        CodeManagement +
                        WordManagement +
                        ThreadManagement
{
    /// Add a new path to the search path list.  This path will be checked to make sure that it
    /// exists.
    fn add_search_path(&mut self, path: &String) -> error::Result<()>;

    /// Add the parent directory for a file to the search paths.  This way if a file includes other
    /// files within it's directory, they'll be found.
    fn add_search_path_for_file(&mut self, file_path: &String) -> error::Result<()>;

    /// Drop the last added path from the search path list.  It is in this way, the search path list
    /// acts like a stack.
    fn drop_search_path(&mut self) -> error::Result<()>;

    /// Return a list of paths that the interpreter will search when finding files.
    fn search_paths(&self) -> &Vec<String>;


    /// Find a file in the current list of search paths.  If the file is found return the fully
    /// qualified path to the file.
    fn find_file(&self, path: & String) -> error::Result<String>;


    /// The current list of variables known to the interpreter.
    fn variables(&self) -> &VariableList;

    /// The current word dictionary of words known to the interpreter.
    fn dictionary(&self) -> &Dictionary;

    /// The current list of data object definitions known to the interpreter.
    fn structure_definitions(&self) -> &DataDefinitionList;


    /// Reset the interpreter to a prior context state, while also clearing the data stack.  After
    /// reset a new context is created.
    fn reset(&mut self) -> error::Result<()>;
}
