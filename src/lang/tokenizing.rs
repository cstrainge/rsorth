
use std::{ cmp::Ordering,
           fmt::{ self, Debug, Display, Formatter },
           fs::read_to_string,
           hash::{ Hash, Hasher } };
use crate::{ lang::source_buffer::{ SourceBuffer, SourceLocation },
             runtime::{ data_structures::value::Value,
                        error::{ self, ScriptError, script_error_str },
                        interpreter::Interpreter } };



#[derive(Clone, Copy)]
pub enum NumberType
{
    Int(i64),
    Float(f64)
}


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



#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub enum Token
{
    Number(SourceLocation, NumberType),
    String(SourceLocation, String),
    Word(SourceLocation, String)
}


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
    pub fn location(&self) -> &SourceLocation
    {
        match self
        {
            Token::Number(location, _) => location,
            Token::String(location, _) => location,
            Token::Word(location, _)   => location
        }
    }

    pub fn is_number(&self) -> bool
    {
        match self
        {
            Token::Number(_, _) => true,
            _                   => false
        }
    }

    pub fn number(&self, interpreter: &mut dyn Interpreter) -> error::Result<&NumberType>
    {
        match self
        {
            Token::Number(_, number) => Ok(number),
            _                        => script_error_str(interpreter, "Token is not a number.")
        }
    }

    pub fn is_textual(&self) -> bool
    {
        match self
        {
            Token::String(_, _) => true,
            Token::Word(_, _)   => true,
            _                   => false
        }
    }

    pub fn text(&self, interpreter: &mut dyn Interpreter) -> error::Result<&String>
    {
        match self
        {
            Token::String(_, text) => Ok(text),
            Token::Word(_, text)   => Ok(text),
            _                      => script_error_str(interpreter, "Token is not textual.")
        }
    }
    pub fn is_string(&self) -> bool
    {
        match self
        {
            Token::String(_, _) => true,
            _                   => false
        }
    }

    pub fn string(&self, interpreter: &mut dyn Interpreter) -> error::Result<&String>
    {
        match self
        {
            Token::String(_, text) => Ok(text),
            _                      => script_error_str(interpreter, "Token is not a string.")
        }
    }

    pub fn is_word(&self) -> bool
    {
        match self
        {
            Token::Word(_, _) => true,
            _                 => false
        }
    }

    pub fn word(&self, interpreter: &mut dyn Interpreter) -> error::Result<&String>
    {
        match self
        {
            Token::Word(_, word) => Ok(word),
            _                    => script_error_str(interpreter, "Token is not a word.")
        }
    }
}



fn is_whitespace(next: &char) -> bool
{
    *next == ' ' || *next == '\t' || *next == '\r' || *next == '\n'
}


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


fn process_literal(location: &SourceLocation, buffer: &mut SourceBuffer) -> error::Result<char>
{
    let next = buffer.next().unwrap();

    assert!(next == '\\');

    match buffer.next()
    {
        Some('n') => Ok('\n'),
        Some('r') => Ok('\r'),
        Some('t') => Ok('\t'),
        Some('0') =>
            {
                let number_str = String::new();

                if let Ok(number) = number_str.parse::<char>()
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
        Some(next) => Ok(next),
        None => ScriptError::new_as_result(Some(location.clone()),
                                           "Unexpected end of file in string literal.".to_string(),
                                           None)
    }
}


fn process_multi_line_string(location: &SourceLocation,
                             buffer: &mut SourceBuffer) -> error::Result<String>
{
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

    fn append_newlines(text: &mut String, count: usize)
    {
        for _ in 0..count
        {
            text.push('\n');
        }
    }

    let next = buffer.next().unwrap();
    assert!(next == '*');

    skip_whitespace(buffer);

    let target_column = buffer.location().column();
    let mut text = String::new();

    while let Some(next) = buffer.next()
    {
        match next
        {
            '*' =>
                {
                    if let Some(quote) = buffer.peek_next()
                    {
                        if quote == '"'
                        {
                            let _ = buffer.next();
                            break;
                        }
                        else
                        {
                            text.push('*');
                        }
                    }
                    else
                    {
                        ScriptError::new_as_result(Some(location.clone()),
                                            "Unexpected end of file in string literal.".to_string(),
                                            None)?;
                    }
                },

            '\\' => text.push(process_literal(location, buffer)?),

            '\n' =>
                {
                    text.push('\n');

                    let start_line = buffer.location().line();

                    skip_whitespace_until_column(&location, buffer, target_column)?;

                    let current_line = buffer.location().line();

                    if current_line > start_line
                    {
                        append_newlines(&mut text, current_line - start_line);
                    }
                }

            _ =>
                {
                    text.push(next);
                }
        }
    }

    Ok(text)
}


fn process_string(buffer: &mut SourceBuffer) -> error::Result<( SourceLocation, String )>
{
    let next = buffer.next().unwrap();
    let location = buffer.location().clone();
    let mut text = String::new();

    assert!(next == '"');

    if buffer.peek_next() == Some('*')
    {
        text = process_multi_line_string(&location, buffer)?;
    }
    else
    {
        while let Some(next) = buffer.peek_next() && next != '"'
        {
            match next
            {
                '\n' => ScriptError::new_as_result(Some(location.clone()),
                                               "Unexpected new line in string literal.".to_string(),
                                               None)?,
                '\\' => text.push(process_literal(&location, buffer)?),
                _    => text.push(buffer.next().unwrap())

            }
        }

        let result = buffer.next();

        if result.is_none()
        {
            ScriptError::new_as_result(Some(location.clone()),
                                       "Unexpected end of file in string literal.".to_string(),
                                       None)?;
        }

        assert!(result.unwrap() == '"');
    }

    Ok(( location, text ))
}


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


fn to_numeric(text: &String) -> Option<NumberType>
{
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

    result
}


pub fn tokenize_from_source(path: &String, source: &String) -> error::Result<TokenList>
{
    let mut buffer = SourceBuffer::new(path, source);
    let mut token_list = TokenList::new();

    while let Some(next) = buffer.peek_next()
    {
        if is_whitespace(&next)
        {
            skip_whitespace(&mut buffer);
            continue;
        }

        let mut is_string = false;

        let location: SourceLocation;
        let text: String;

        if next == '"'
        {
            is_string = true;
            ( location, text ) = process_string(&mut buffer)?;
        }
        else
        {
            ( location, text ) = process_until_whitespace(&mut buffer);
        }

        let next_token = match text
            {
                _ if is_string => Token::String(location, text),
                _ if is_number(&text) =>
                    {
                        if let Some(number) = to_numeric(&text)
                        {
                            Token::Number(location, number)
                        }
                        else
                        {
                            Token::Word(location, text)
                        }
                    },
                _ => Token::Word(location, text)
            };

        token_list.push(next_token);
    }

    Ok(token_list)
}


pub fn tokenize_from_file(path: &String) -> error::Result<TokenList>
{
    let result = read_to_string(path);

    if let Err(error) = &result
    {
        ScriptError::new_as_result(None,
                                   format!("Could not read file {}: {}", path, error),
                                   None)?;
    }

    tokenize_from_source(path, &result.unwrap())
}
