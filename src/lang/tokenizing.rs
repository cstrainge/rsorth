
use std::{ cmp::Ordering,
           fmt::{ self, Debug, Display, Formatter },
           fs::read_to_string,
           hash::{ Hash, Hasher } };
use crate::{ lang::source_buffer::{ SourceBuffer, SourceLocation },
             runtime::{ data_structures::value::Value,
                        error::{ self, ScriptError, script_error_str },
                        interpreter::Interpreter } };



/// A number token can be either an integer or a floating point literal.
#[derive(Clone, Copy)]
pub enum NumberType
{
    /// We're holding an integer value.
    Int(i64),

    /// We're holding a floating point value.
    Float(f64)
}


/// We have this implementation here even though we could possibly be holding a floating point
/// value.  This potentially invalidates the Eq implementation, but this is needed so that we can
/// use Values in hash maps.
impl Eq for NumberType {}


impl PartialEq for NumberType
{
    fn eq(&self, other: &Self) -> bool
    {
        match ( self, other )
        {
            ( NumberType::Int(a), NumberType::Int(b) )     => a == b,
            ( NumberType::Float(a), NumberType::Float(b) ) => a == b,

            ( NumberType::Float(a), NumberType::Int(b) )   => a == &(*b as f64),
            ( NumberType::Int(a), NumberType::Float(b) )   => &(*a as f64) == b
        }
    }
}


impl PartialOrd for NumberType
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        match ( self, other )
        {
            ( NumberType::Int(a), NumberType::Int(b) )     => a.partial_cmp(b),
            ( NumberType::Float(a), NumberType::Float(b) ) => a.partial_cmp(b),

            ( NumberType::Float(a), NumberType::Int(b) )   => a.partial_cmp(&(*b as f64)),
            ( NumberType::Int(a), NumberType::Float(b) )   => (*a as f64).partial_cmp(b)
        }
    }
}


impl Hash for NumberType
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        match self
        {
            NumberType::Int(num) => num.hash(state),
            NumberType::Float(num) => num.to_bits().hash(state)
        }
    }
}


/// Print the value of the held number.
impl Display for NumberType
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match self
        {
            NumberType::Int(num) => write!(f, "{}", num),
            NumberType::Float(num) => write!(f, "{}", num)
        }
    }
}


/// Print the value of the held number as well as an indicator of which variant we're holding for
/// debugging purposes.
impl Debug for NumberType
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match self
        {
            NumberType::Int(num) => write!(f, "{} i", num),
            NumberType::Float(num) => write!(f, "{} f", num)
        }
    }
}



/// A token is a simple unit of the language.  Due to the language's simplicity we only have three
/// possibilities.  The token can only be a number, a string, or a word.
///
/// The token also holds the location in the original source code where it was found.
///
/// Because a token can be held by a Value we need to implement the Hash and Eq traits.  This
/// potentially invalidates the Eq implementation because we could be holding a floating point
/// value.  However, this is needed so that we can use Values in hash maps.
///
/// It is important to note this in the user documentation that floating point values should not be
/// used as keys in hash maps.
#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub enum Token
{
    /// Can be either an integer or a floating point value.
    Number(SourceLocation, NumberType),

    /// A single line or multi-line string literal.
    String(SourceLocation, String),

    /// A word in the language to be executed.
    Word(SourceLocation, String)
}



/// A list of tokens found in the source code.
pub type TokenList = Vec<Token>;



impl Hash for Token
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        match self
        {
            Token::Number(location, value) =>
                {
                    location.hash(state);
                    value.hash(state);
                },

            Token::String(location, value) =>
                {
                    location.hash(state);
                    value.hash(state);
                },

            Token::Word(location, value) =>
                {
                    location.hash(state);
                    value.hash(state);
                }

        }
    }
}


/// Make sure that the tokens are nicely printable for debugging purposes.
impl Display for Token
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match self
        {
            Token::Number(_, num)    => write!(f, "{}", num),
            Token::String(_, string) => write!(f, "{}", string),
            Token::Word(_, string)   => write!(f, "{}", string)
        }
    }
}


/// Make sure that the tokens are nicely printable for debugging purposes.  We can include extra
/// information such as the original location and extra formatting for the string literals.
impl Debug for Token
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match self
        {
            Token::Number(location, num)    => write!(f, "{}: {:?}", location, num),
            Token::String(location, string) => write!(f, "{}: {}", location,
                                                                   Value::stringify(string)),
            Token::Word(location, string)   => write!(f, "{}: {}", location, string)
        }
    }
}



impl Token
{
    /// Get the token's location in the original source text.
    pub fn location(&self) -> &SourceLocation
    {
        match self
        {
            Token::Number(location, _) => location,
            Token::String(location, _) => location,
            Token::Word(location, _)   => location
        }
    }

    /// Check if the token is a number.
    pub fn is_number(&self) -> bool
    {
        match self
        {
            Token::Number(_, _) => true,
            _                   => false
        }
    }

    /// Get the number value of the token, or error if it isn't a number token.
    pub fn number(&self, interpreter: &mut dyn Interpreter) -> error::Result<&NumberType>
    {
        match self
        {
            Token::Number(_, number) => Ok(number),
            _                        => script_error_str(interpreter, "Token is not a number.")
        }
    }

    /// Check if the token is either a word or a string literal.
    pub fn is_textual(&self) -> bool
    {
        match self
        {
            Token::String(_, _) => true,
            Token::Word(_, _)   => true,
            _                   => false
        }
    }

    /// Get the text value of the token, be it a word or a string literal.  Error out if it is a
    /// number token.
    pub fn text(&self, interpreter: &mut dyn Interpreter) -> error::Result<&String>
    {
        match self
        {
            Token::String(_, text) => Ok(text),
            Token::Word(_, text)   => Ok(text),
            _                      => script_error_str(interpreter, "Token is not textual.")
        }
    }

    /// Check if the token is a string literal.
    pub fn is_string(&self) -> bool
    {
        match self
        {
            Token::String(_, _) => true,
            _                   => false
        }
    }

    /// Get the string value of the token, or error if it is a number or word.
    pub fn string(&self, interpreter: &mut dyn Interpreter) -> error::Result<&String>
    {
        match self
        {
            Token::String(_, text) => Ok(text),
            _                      => script_error_str(interpreter, "Token is not a string.")
        }
    }

    /// Check if the token is a word.
    pub fn is_word(&self) -> bool
    {
        match self
        {
            Token::Word(_, _) => true,
            _                 => false
        }
    }

    /// Get the word text or error if it is a string literal or a number token.
    pub fn word(&self, interpreter: &mut dyn Interpreter) -> error::Result<&String>
    {
        match self
        {
            Token::Word(_, word) => Ok(word),
            _                    => script_error_str(interpreter, "Token is not a word.")
        }
    }
}



/// Check if the given character is considered whitespace.
fn is_whitespace(next: &char) -> bool
{
    *next == ' ' || *next == '\t' || *next == '\r' || *next == '\n'
}

/// Skip over whitespace in the text.  Stopping only at either the end of the buffer or the next
/// non-whitespace character.
fn skip_whitespace(buffer: &mut SourceBuffer)
{
    while let Some(next) = buffer.peek_next()
    {
        if !is_whitespace(&next)
        {
            break;
        }

        let _ = buffer.next();
    }
}

/// Process an escape sequence in a string literal.  This can be a newline, carriage return, tab, or
/// a numeric literal for a character.
fn process_literal(location: &SourceLocation, buffer: &mut SourceBuffer) -> error::Result<char>
{
    let next = buffer.next().unwrap();

    assert!(next == '\\');

    match buffer.next()
    {
        // Perform a simple translation of the escape sequence.
        Some('n') => Ok('\n'),
        Some('r') => Ok('\r'),
        Some('t') => Ok('\t'),

        // Parse a numeric literal for the character.  This can be single or multiple digits.
        Some('0') =>
            {
                let mut number_str = String::new();

                while    let Some(next) = buffer.peek_next()
                      && next.is_digit(10)
                {
                    number_str.push(buffer.next().unwrap());
                }

                if let Ok(number) = number_str.parse::<u8>()
                {
                    Ok(number as char)
                }
                else
                {
                    ScriptError::new_as_result(Some(location.clone()),
                                  format!("Failed to parse numeric literal from '{}'.", number_str),
                                  None)
                }
            },

        // The escape was on a non-special character so just pass it through without translation.
        Some(next) => Ok(next),

        // Looks like we hit the end of the buffer while processing a string.
        None => ScriptError::new_as_result(Some(location.clone()),
                                           "Unexpected end of file in string literal.".to_string(),
                                           None)
    }
}

/// Process a multi-line string literal.  This can contain new lines and escape sequences.  Extra
/// whitespace is removed from the beginning of each line.  This way the string can be formatted
/// nicely in the source code.
fn process_multi_line_string(location: &SourceLocation,
                             buffer: &mut SourceBuffer) -> error::Result<String>
{
    // Helper for skipping extra whitespace at the beginning of each line.  If there is no text
    // on a given line it is skipped entirely.
    fn skip_whitespace_until_column(location: &SourceLocation,
                                    buffer: &mut SourceBuffer,
                                    target_column: usize) -> error::Result<()>
    {
        while    let Some(next) = buffer.peek_next()
              && is_whitespace(&next)
              && buffer.location().column() < target_column
        {
            let _ = buffer.next();
        }

        if let None = buffer.peek_next()
        {
            ScriptError::new_as_result(Some(location.clone()),
                                       "Unexpected end of file in string literal.".to_string(),
                                       None)?;
        }

        Ok(())
    }

    // Append newlines for skipped empty lines.
    fn append_newlines(text: &mut String, count: usize)
    {
        for _ in 0..count
        {
            text.push('\n');
        }
    }

    // We expect that the " has already be processed and that we need to consume the following *.
    let next = buffer.next().unwrap();
    assert!(next == '*');

    // Skip over any whitespace at the beginning of the string.  Using the location of the first
    // textual character to calibrate what we will consider the beginning of the actual line of
    // text.  This way we can remove any extra whitespace at the beginning of each line while
    // allowing for any extra indentation the user may want to add.
    skip_whitespace(buffer);

    let target_column = buffer.location().column();
    let mut text = String::new();

    // Keep going until we either hit the end of the buffer or the closing *" pair.
    while let Some(next) = buffer.next()
    {
        match next
        {
            // We found the * but did we find the "?
            '*' =>
                {
                    if let Some(quote) = buffer.peek_next()
                    {
                        // We're at the end of the string.
                        if quote == '"'
                        {
                            let _ = buffer.next();
                            break;
                        }
                        else
                        {
                            // Looks like a stray * so we'll just add it to the text.
                            text.push('*');
                        }
                    }
                    else
                    {
                        // Make sure we didn't hit the end of the buffer while looking for the ".
                        ScriptError::new_as_result(Some(location.clone()),
                                            "Unexpected end of file in string literal.".to_string(),
                                            None)?;
                    }
                },

            // Process the escape sequence.
            '\\' => text.push(process_literal(location, buffer)?),

            // Process the new line skipping any extra whitespace until we hit the target column.
            '\n' =>
                {
                    text.push('\n');

                    // Keep track of the starting line so that we can add newlines for skipped empty
                    // lines.  Then start skipping until we find something useful or we hit the
                    // target column.
                    let start_line = buffer.location().line();

                    skip_whitespace_until_column(&location, buffer, target_column)?;

                    // If we skipped any empty lines then we need to backfill the newlines.
                    let current_line = buffer.location().line();

                    if current_line > start_line
                    {
                        append_newlines(&mut text, current_line - start_line);
                    }
                }

            // Just add the character to the text.
            _ =>
                {
                    text.push(next);
                }
        }
    }

    // Looks like we found the closing *" pair so we can return the text.
    Ok(text)
}

/// Process a single line string literal.  This can contain escape sequences but not new lines.
/// If an opening "* is found then we process as a multi-line string literal which follows different
/// rules.
fn process_string(buffer: &mut SourceBuffer) -> error::Result<( SourceLocation, String )>
{
    let next = buffer.next().unwrap();
    let location = buffer.location().clone();
    let mut text = String::new();

    // Expect the opening ".
    assert!(next == '"');

    // Check for the start of a multi-line string literal.
    if buffer.peek_next() == Some('*')
    {
        text = process_multi_line_string(&location, buffer)?;
    }
    else
    {
        // This is a single line literal keep going until we hit the end of the buffer or we find
        // the closing ".
        while let Some(next) = buffer.peek_next() && next != '"'
        {
            match next
            {
                // These are not allowed in a single line literal.
                '\n' => ScriptError::new_as_result(Some(location.clone()),
                                               "Unexpected new line in string literal.".to_string(),
                                               None)?,

                // Process the escape sequence.
                '\\' => text.push(process_literal(&location, buffer)?),

                // Just add the character to the text.
                _    => text.push(buffer.next().unwrap())

            }
        }

        // Make sure we found the closing ", otherwise we hit the end of the buffer.
        let result = buffer.next();

        if result.is_none()
        {
            ScriptError::new_as_result(Some(location.clone()),
                                       "Unexpected end of file in string literal.".to_string(),
                                       None)?;
        }

        assert!(result.unwrap() == '"');
    }

    // Return either version of the string literal's text and the location where it was found.
    Ok(( location, text ))
}

/// Pull text out of the buffer until we hit a whitespace character.  This is used to process words.
/// Words can contain any character except whitespace.
fn process_until_whitespace(buffer: &mut SourceBuffer) -> ( SourceLocation, String )
{
    let location = buffer.location().clone();
    let mut text = String::new();

    while let Some(next) = buffer.peek_next() && !is_whitespace(&next)
    {
        let next = buffer.next().unwrap();
        text.push(next);
    }

    ( location, text )
}

/// Does it look like we're dealing with a numeric literal?
fn is_number(text: &String) -> bool
{
    if text.is_empty()
    {
        return false;
    }

    if    text.starts_with("0x")
       || text.starts_with("0b")
    {
        return true;
    }

    text.chars().all(|c|    c.is_digit(16)
                         || c == '.'
                         || c == '-'
                         || c == 'e'
                         || c == 'E'
                         || c == '_' )
}


/// Attempt to convert the text into a numeric literal.  This can be either an integer or floating
/// point number.  We also support hexadecimal and binary literals, and using _ as a separator for
/// readability.
fn to_numeric(text: &String) -> Option<NumberType>
{
    // If the attempt at parsing the number fails then we return None.
    fn check_numeric_error<T, E>(result: &Result<T, E>) -> Option<()>
        where
            E: Display
    {
        if let Err(_) = result
        {
            return None;
        }

        Some(())
    }

    // Check for the number literal type and process accordingly.
    let result =
        if text.starts_with("0x")
        {
            let result = i64::from_str_radix(&text[2..].replace("_", ""), 16);

            check_numeric_error(&result)?;
            Some(NumberType::Int(result.ok()?))
        }
        else if text.starts_with("0b")
        {
            let result = i64::from_str_radix(&text[2..].replace("_", ""), 2);

            check_numeric_error(&result)?;
            Some(NumberType::Int(result.ok()?))
        }
        else if text.contains('.')
        {
            let result = text.replace("_", "").parse();

            check_numeric_error(&result)?;
            Some(NumberType::Float(result.ok()?))
        }
        else
        {
            let result = text.replace("_", "").parse();

            check_numeric_error(&result)?;
            Some(NumberType::Int(result.ok()?))
        };

    // We either have a number or nothing at this point.
    result
}



/// Tokenize the source code from a string.
pub fn tokenize_from_source(path: &String, source: &String) -> error::Result<TokenList>
{
    let mut buffer = SourceBuffer::new(path, source);
    let mut token_list = TokenList::new();

    // Keep going until we hit the end of the buffer or error out.
    while let Some(next) = buffer.peek_next()
    {
        // Skip over any whitespace.
        if is_whitespace(&next)
        {
            skip_whitespace(&mut buffer);
            continue;
        }

        // We'll extract the next token from the buffer.
        let mut is_string = false;

        let location: SourceLocation;
        let text: String;

        // Is this a string?
        if next == '"'
        {
            is_string = true;
            ( location, text ) = process_string(&mut buffer)?;
        }
        else
        {
            // No, this is a word or a number, tbd later.
            ( location, text ) = process_until_whitespace(&mut buffer);
        }

        // We'll determine what type of token we have based on the found text and string flag.
        let next_token = match text
            {
                // It was definitely a string literal.
                _ if is_string => Token::String(location, text),

                // It could be a number or a word...
                _ if is_number(&text) =>
                    {
                        // Try it as a number first, otherwise it's a word.
                        if let Some(number) = to_numeric(&text)
                        {
                            Token::Number(location, number)
                        }
                        else
                        {
                            Token::Word(location, text)
                        }
                    },

                // It was definitely a word.
                _ => Token::Word(location, text)
            };

        // Add the new token to the list.
        token_list.push(next_token);
    }

    // Looks like we've hit the end of the buffer without finding any errors.
    Ok(token_list)
}

/// Load the code from a file and then tokenize it.
pub fn tokenize_from_file(path: &String) -> error::Result<TokenList>
{
    // Just read the whole file into a string.
    let result = read_to_string(path);

    // Check if the read was successful.
    if let Err(error) = &result
    {
        ScriptError::new_as_result(None,
                                   format!("Could not read file {}: {}", path, error),
                                   None)?;
    }

    // Tokenize the source code and return the result.
    tokenize_from_source(path, &result.unwrap())
}
