
use core::str::Chars;
use std::{ fmt::{ self, Display, Formatter },
           hash::{ Hash, Hasher } };



#[derive(Clone)]
pub struct SourceLocation
{
    path: String,
    line: u32,
    column: u32
}


impl PartialEq for SourceLocation
{
    fn eq(&self, other: &Self) -> bool
    {
        (self.path == other.path) && (self.line == other.line) && (self.column == other.column)
    }
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


impl Display for SourceLocation
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error>
    {
        write!(formatter, "{} ({}, {})", self.path, self.line, self.column)
    }
}


impl SourceLocation
{
    pub fn new() -> SourceLocation
    {
        SourceLocation { path: "unspecified".to_string(), line: 1, column: 1 }
    }

    pub fn new_from_path(path: &String) -> Self
    {
        SourceLocation { path: path.clone(), line: 1, column: 1 }
    }

    pub fn new_from_info(path: &String, line: u32, column: u32) -> Self
    {
        SourceLocation { path: path.clone(), line, column }
    }

    pub fn path(&self) -> &String
    {
        &self.path
    }

    pub fn line(&self) -> u32
    {
        self.line
    }

    pub fn column(&self) -> u32
    {
        self.column
    }
}



pub struct SourceBuffer<'a>
{
    chars: Chars<'a>,
    location: SourceLocation,
    current: Option<char>
}


impl<'a> SourceBuffer<'a>
{
    pub fn new(path: &String, source: &'a String) -> Self
    {
        SourceBuffer
        {
            chars: source.chars(),
            location: SourceLocation::new_from_path(path),
            current: None
        }
    }

    pub fn location(&self) -> &SourceLocation
    {
        &self.location
    }

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
