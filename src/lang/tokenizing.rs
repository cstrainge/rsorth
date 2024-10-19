
use std::{ fmt::{ self, Debug, Display, Formatter },
           fs::read_to_string,
           hash::{ Hash, Hasher } };
use crate::{ lang::source_buffer::{ SourceBuffer, SourceLocation },
             runtime::error::{self, ScriptError} };



#[derive(Clone, Copy)]
pub enum NumberType
{
    Int(i64),
    Float(f64)
}


impl PartialEq for NumberType
{
    fn eq(&self, other: &Self) -> bool
    {
        match ( self, other )
        {
            ( NumberType::Int(a), NumberType::Int(b) ) => a == b,
            ( NumberType::Float(a), NumberType::Float(b) ) => a == b,

            ( NumberType::Float(a), NumberType::Int(b) ) => a == &(*b as f64),
            ( NumberType::Int(a), NumberType::Float(b) ) => &(*a as f64) == b
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



#[derive(Clone)]
pub enum Token
{
    Number(SourceLocation, NumberType),
    String(SourceLocation, String),
    Word(SourceLocation, String)
}


type TokenList = Vec<Token>;


impl PartialEq for Token
{
    fn eq(&self, other: &Self) -> bool
    {
        match ( self, other )
        {
            ( Token::Number(l_a, v_a), Token::Number(l_b, v_b) ) => (l_a == l_b) && (v_a == v_b),
            ( Token::String(l_a, v_a), Token::String(l_b, v_b) ) => (l_a == l_b) && (v_a == v_b),
            ( Token::Word(l_a, v_a),   Token::Word(l_b, v_b)   ) => (l_a == l_b) && (v_a == v_b),
            _ => false
        }
    }
}


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
            Token::String(location, string) => write!(f, "{}: {:?}", location, string),
            Token::Word(location, string)   => write!(f, "{}: {:?}", location, string)
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
                                    target_column: u32) -> error::Result<()>
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

    fn append_newlines(text: &mut String, count: u32)
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

    while let Some(next) = buffer.peek_next()
    {
        match next
        {
            '*' =>
                {
                    if let Some(next) = buffer.peek_next()
                    {
                        if next == '"'
                        {
                            let _ = buffer.next();
                            break;
                        }
                        else
                        {
                            text.push(next);
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

            _ => text.push(next)
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
    let _ = buffer.next();

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

    text.chars().all(|c|    c.is_digit(16)
                         || c == '.'
                         || c == '-'
                         || c == 'e'
                         || c == 'E'
                         || c == '_' )
}


fn to_numeric(location: &SourceLocation, text: &String) -> error::Result<NumberType>
{
    fn check_numeric_error<T, E>(location: &SourceLocation,
                                 result: &Result<T, E>,
                                 original: &String) -> error::Result<()>
        where
            E: Display
    {
        if let Err(err) = result
        {
            let message = format!("Could not parse numeric value from '{}', {}.", original, err);
            return ScriptError::new_as_result(Some(location.clone()), message, None);
        }

        Ok(())
    }

    let numeric;

    if text.find('.').is_some()
    {
        let result = text.parse();

        check_numeric_error(location, &result, text)?;
        numeric = NumberType::Float(result.unwrap());
    }
    else
    {
        let result = text.parse();

        check_numeric_error(location, &result, text)?;
        numeric = NumberType::Int(result.unwrap());
    }

    Ok(numeric)
}


pub fn tokenize_from_source(path: &String, source: &String) -> error::Result<TokenList>
{
    let mut buffer = SourceBuffer::new(path, source);
    let mut token_list = Vec::new();

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
                        if let Ok(number) = to_numeric(&location, &text)
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
