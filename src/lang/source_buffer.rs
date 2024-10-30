
use core::str::Chars;
use std::{ fmt::{ self,
                  Display,
                  Formatter },
           hash::{ Hash,
                   Hasher } };



/// The location in the source code where a token was found.  This structure is used all over the
/// interpreter to keep track where important things are found in the source code.  This is used
/// extensively in the error reporting.
///
/// This is a read-only structure.  Use the field accessor methods to get the values.
#[derive(Clone, PartialEq, PartialOrd, Eq)]
pub struct SourceLocation
{
    /// Either the path to the file or a description of the source code.  For example code entered
    /// in the REPL will have a tag of "\<repl\>".
    path: String,

    /// The 1 based line number in the source code where the token was found.
    line: usize,

    /// The 1 based column number in the source code where the token was found.
    column: usize
}


impl Hash for SourceLocation
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        self.path.hash(state);
        self.line.hash(state);
        self.column.hash(state);
    }
}


/// Used for error reporting to show where in the source code an error originated.
impl Display for SourceLocation
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error>
    {
        write!(formatter, "{} ({}, {})", self.path, self.line, self.column)
    }
}


impl SourceLocation
{
    /// Crate a new SourceLocation with default values.
    pub fn new() -> SourceLocation
    {
        SourceLocation { path: "unspecified".to_string(), line: 1, column: 1 }
    }

    /// Create a new SourceLocation with the path to the source code.
    pub fn new_from_path(path: &String) -> Self
    {
        SourceLocation { path: path.clone(), line: 1, column: 1 }
    }

    /// Create a new SourceLocation with all of the needed information.  This is useful in
    /// conjunction with the location_here! macro.
    pub fn new_from_info(path: &String, line: usize, column: usize) -> Self
    {
        SourceLocation { path: path.clone(), line, column }
    }

    /// The path to the source code or a meaningful description of the source code.
    pub fn path(&self) -> &String
    {
        &self.path
    }

    /// The 1 based line number in the source code.
    pub fn line(&self) -> usize
    {
        self.line
    }

    /// The 1 based column number in the source code.
    pub fn column(&self) -> usize
    {
        self.column
    }
}



/// Helper macro to get the location of the macro invocation.  This is useful for error reporting
/// that includes locations within the Rust code where important operations are occurring.
#[macro_export]
macro_rules! location_here
{
    () =>
    {
        crate::lang::source_buffer::SourceLocation::new_from_info(&file!().to_string(),
                                                                  line!() as usize,
                                                                  column!() as usize)
    };
}



/// A buffer for processing source code.  This is used by the tokenizer to extract meaningful tokens
/// from the source code.  This buffer acts as a forward only iterator over the code.  As characters
/// are consumed the location of the cursor in that source is maintained.  Thus allowing the
/// tokenizer to keep track of important points in the source code.
///
/// The SourceBuffer only holds a reference to the source code, the code is not copied.  The source
/// code string is expected to outlive the SourceBuffer.
pub struct SourceBuffer<'a>
{
    /// An iterator over the source code being processed.  Because this is a reference to the
    /// original text it is important that the source code outlives the SourceBuffer.
    chars: Chars<'a>,

    /// The logical location of the cursor in the source code.
    location: SourceLocation,

    /// The current character being processed.  This is used to peek at the next character without
    /// consuming it.
    current: Option<char>
}


impl<'a> SourceBuffer<'a>
{
    /// Create a new SourceBuffer with the path to, or meaningful tag for the source code and the
    /// source code itself.
    ///
    /// It is important to note that the source code is not copied.  The SourceBuffer will hold a
    /// reference to the source code.  The code will not be modified and it is expected that the
    /// source code will outlive the SourceBuffer.
    pub fn new(path: &String, source: &'a String) -> Self
    {
        SourceBuffer
        {
            chars: source.chars(),
            location: SourceLocation::new_from_path(path),
            current: None
        }
    }

    /// The location the cursor is at in the source code being processed.
    pub fn location(&self) -> &SourceLocation
    {
        &self.location
    }

    /// Take a peek at the next character in the source code without consuming it.
    pub fn peek_next(&mut self) -> Option<char>
    {
        match self.current
        {
            Some(_) => self.current,
            None =>
                {
                    let next = self.chars.next();

                    self.current = next;
                    next
                }
        }
    }

    /// Get and consume the next character in the source code.
    pub fn next(&mut self) -> Option<char>
    {
        let next: Option<char>;

        match self.current
        {
            Some(_) =>
                {
                    next = self.current;
                    self.current = None;
                },

            None => next = self.chars.next()
        }

        if let Some(next_char) = next
        {
            self.increment_location(next_char);
        }

        next
    }

    /// Ok, the source buffer is allowed to modify the location.  This is because the location is
    /// based on the source code and the source code is being managed by the source buffer.
    ///
    /// Increment the location based on the next character.  Advance one column for regular
    /// characters.  Reset the colum to 1 and increment the line for new line characters.
    fn increment_location(&mut self, next: char)
    {
        if next == '\n'
        {
            self.location.line += 1;
            self.location.column = 1;
        }
        else
        {
            self.location.column += 1;
        }
    }
}
