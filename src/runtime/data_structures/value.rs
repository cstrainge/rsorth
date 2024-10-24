
use std::{ cell::RefCell,
           fmt::{ self, Display, Formatter },
           hash::{ Hash, Hasher } };
use crate::{ lang::tokenizing::{ NumberType, Token },
             runtime::{ data_structures::{ data_object::DataObjectPtr,
                                           value_vec::{ ValueVec, ValueVecPtr } },
                        error::{ self, script_error },
                        interpreter::Interpreter } };



#[derive(Clone)]
pub enum Value
{
    None,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Vec(ValueVecPtr),
    DataObject(DataObjectPtr),
    Token(Token)
}



pub trait ToValue
{
    fn to_value(&self) -> Value;
}



impl ToValue for &String
{
    fn to_value(&self) -> Value
    {
        let string = (*self).clone();
        Value::String(string)
    }
}



impl Default for Value
{
    fn default() -> Value
    {
        Value::None
    }
}


impl PartialEq for Value
{
    fn eq(&self, other: &Value) -> bool
    {
        if Value::both_are_numeric(self, other)
        {
            if Value::either_is_float(self, other)
            {
                let a = self.get_float_val().unwrap();
                let b = self.get_float_val().unwrap();

                a == b
            }
            else if Value::either_is_int(self, other)
            {
                let a = self.get_int_val().unwrap();
                let b = self.get_int_val().unwrap();

                a == b
            }
            else if Value::either_is_bool(self, other)
            {
                let a = self.get_bool_val().unwrap();
                let b = other.get_bool_val().unwrap();

                a == b
            }
            else
            {
                false
            }
        }
        else if self.is_string_like() && other.is_string_like()
        {
            let a = self.get_string_val().unwrap();
            let b = other.get_string_val().unwrap();

            a == b
        }
        else
        {
            match ( self, other )
            {
                ( Value::Vec(a),        Value::Vec(b)        ) => *a.borrow() == *b.borrow(),
                ( Value::DataObject(a), Value::DataObject(b) ) => *a.borrow() == *b.borrow(),
                ( Value::Token(a),      Value::Token(b)      ) => a == b,
                _                                              => false
            }
        }
    }
}


impl Hash for Value
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        match self
        {
            Value::None              => 0.hash(state),
            Value::Int(value)        => value.hash(state),
            Value::Float(value)      => value.to_bits().hash(state),
            Value::Bool(value)       => value.hash(state),
            Value::String(value)     => value.hash(state),
            Value::Vec(value)        => value.borrow().hash(state),
            Value::DataObject(value) => value.borrow().hash(state),
            Value::Token(value)      => value.hash(state)
        }
    }
}


impl Display for Value
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match self
        {
            Value::None              => write!(f, "none"),
            Value::Int(value)        => write!(f, "{}", value),
            Value::Float(value)      => write!(f, "{}", value),
            Value::Bool(value)       => write!(f, "{}", value),
            Value::String(value)     => write!(f, "{}", value),
            Value::Vec(value)        => write!(f, "{}", value.borrow()),
            Value::DataObject(value) => write!(f, "{}", value.borrow()),
            Value::Token(value)      => write!(f, "{}", value)
        }
    }
}



macro_rules! value_conversion
{
    ($data_type:ty , $variant:ident , $as_ident:ident) =>
    {
        impl Value
        {
            pub fn $as_ident(&self, interpreter: &dyn Interpreter) -> error::Result<&$data_type>
            {
                match self
                {
                    Value::$variant(value) => Ok(value),
                    _ => script_error(interpreter,
                                      format!("Value could not be converted to {}",
                                              stringify!($data_type)))
                }
            }
        }

        impl ToValue for $data_type
        {
            fn to_value(&self) -> Value
            {
                Value::$variant(self.clone())
            }
        }

        impl From<$data_type> for Value
        {
            fn from(original: $data_type) -> Value
            {
                original.to_value()
            }
        }

        impl From<Value> for $data_type
        {
            fn from(original: Value) -> $data_type
            {
                if let Value::$variant(contained_value) = original
                {
                    return contained_value;
                }

                panic!("Could not automatically convert from a Value to a {}", stringify!($type));
            }
        }
    };
}



impl ToValue for NumberType
{
    fn to_value(&self) -> Value
    {
        match self
        {
            NumberType::Int(value)   => Value::Int(*value),
            NumberType::Float(value) => Value::Float(*value)
        }
    }
}



impl ToValue for usize
{
    fn to_value(&self) -> Value
    {
        Value::Int(*self as i64)
    }
}



impl<T> From<Vec<T>> for Value
    where
        T: ToValue
{
    fn from(vec: Vec<T>) -> Value
    {
        let new_vec: Vec<Value> = vec.iter().map(|item| item.to_value()).collect();
        Value::Vec(ValueVec::from_vec(new_vec))
    }
}



impl<T> From<&Vec<T>> for Value
    where
        T: ToValue
{
    fn from(vec: &Vec<T>) -> Value
    {
        let new_vec: Vec<Value> = vec.iter().map(|item| item.to_value()).collect();
        Value::Vec(ValueVec::from_vec(new_vec))
    }
}



value_conversion!(i64,           Int,        as_int);
value_conversion!(f64,           Float,      as_float);
value_conversion!(bool,          Bool,       as_bool);
value_conversion!(String,        String,     as_string);
value_conversion!(ValueVecPtr,   Vec,        as_vec);
value_conversion!(DataObjectPtr, DataObject, as_data_object);
value_conversion!(Token,         Token,      as_token);



macro_rules! is_variant
{
    ($name:ident , $either_name:ident , $variant:ident) =>
    {
        pub fn $name(&self) -> bool
        {
            if let Value::$variant(ref _value) = self
            {
                true
            }
            else
            {
                false
            }
        }

        pub fn $either_name(a: &Value, b: &Value) -> bool
        {
            a.$name() || b.$name()
        }
    };
}



impl Value
{
    is_variant!(is_int,         either_is_int,         Int);
    is_variant!(is_float,       either_is_float,       Float);
    is_variant!(is_bool,        either_is_bool,        Bool);
    is_variant!(is_string,      either_is_string,      String);
    is_variant!(is_vec,         either_is_vec,         Vec);
    is_variant!(is_data_object, either_is_data_object, DataObject);

    pub fn is_numeric(&self) -> bool
    {
        match self
        {
            Value::None                 => true,
            Value::Int(_)               => true,
            Value::Float(_)             => true,
            Value::Bool(_)              => true,
            Value::Token(token) =>
                match token
                {
                    Token::Number(_, _) => true,
                    _                   => false
                }
            _                           => false
        }
    }

    pub fn both_are_numeric(a: &Value, b: &Value) -> bool
    {
        a.is_numeric() && b.is_numeric()
    }

    pub fn is_string_like(&self) -> bool
    {
        match self
        {
            Value::String(_)            => true,
            Value::Token(token) =>
                match token
                {
                    Token::String(_, _) => true,
                    Token::Word(_, _)   => true,
                    _                   => false
                }
            _                           => false
        }
    }

    pub fn get_string_val(&self) -> Option<String>
    {
        match self
        {
            Value::String(value)            => Some(value.clone()),
            Value::Token(token) =>
                match token
                {
                    Token::String(_, value) => Some(value.clone()),
                    Token::Word(_, word)    => Some(word.clone()),
                    _                       => None
                }
            _                               => None
        }
    }

    pub fn get_bool_val(&self) -> Option<bool>
    {
        match self
        {
            Value::None         => Some(false),
            Value::Int(value)   => Some(*value != 0),
            Value::Float(value) => Some(*value != 0.0),
            Value::Bool(value)  => Some(*value),
            _                   => None
        }
    }

    pub fn get_int_val(&self) -> Option<i64>
    {
        match self
        {
            Value::None                              => Some(0),
            Value::Int(value)                        => Some(*value),
            Value::Float(value)                      => Some(*value as i64),
            Value::Bool(value)                       => Some(if *value { 1 } else { 0 }),
            Value::Token(token) =>
                match token
                {
                    Token::Number(_, num_type) =>
                        match num_type
                        {
                            NumberType::Int(value)   => Some(*value),
                            NumberType::Float(value) => Some(*value as i64)
                        }
                    _                                => None
                }
            _                                        => None
        }
    }

    pub fn get_float_val(&self) -> Option<f64>
    {
        match self
        {
            Value::None                              => Some(0.0),
            Value::Int(value)                        => Some(*value as f64),
            Value::Float(value)                      => Some(*value),
            Value::Bool(value)                       => Some(if *value { 1.0 } else { 0.0 }),
            Value::Token(token) =>
                match token
                {
                    Token::Number(_, num_type) =>
                        match num_type
                        {
                            NumberType::Int(value)   => Some(*value as f64),
                            NumberType::Float(value) => Some(*value)
                        }
                    _                                => None
                }
            _                                        => None
        }
    }
}



impl Value
{
    pub fn stringify(text: &String) -> String
    {
        let mut result = String::new();

        result.push('"');

        for character in text.chars()
        {
            match character
            {
                '"'  => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\\' => result.push_str("\\"),
                _    => result.push(character)
            }
        }

        result.push('"');

        result
    }

}



pub trait DeepClone
{
    fn deep_clone(&self) -> Value;
}



impl DeepClone for Value
{
    fn deep_clone(&self) -> Value
    {
        match self
        {
            Value::None              => Value::None,
            Value::Int(value)        => Value::Int(*value),
            Value::Float(value)      => Value::Float(*value),
            Value::Bool(value)       => Value::Bool(*value),
            Value::String(value)     => Value::String(value.clone()),
            Value::Vec(value)        => value.deep_clone(),
            Value::DataObject(value) => value.deep_clone(),
            Value::Token(value)      => Value::Token(value.clone())
        }
    }
}



thread_local!
{
    static VALUE_FORMAT_INDENT: RefCell<usize> = RefCell::new(0);
}


pub fn value_format_indent() -> usize
{
    let mut indent: usize = 0;

    VALUE_FORMAT_INDENT.with(|value|
        {
            indent = *value.borrow();
        });

    indent
}


pub fn value_format_indent_inc()
{
    VALUE_FORMAT_INDENT.with(|value|
        {
            *value.borrow_mut() += 4;
        });
}


pub fn value_format_indent_dec()
{
    VALUE_FORMAT_INDENT.with(|value|
        {
            *value.borrow_mut() -= 4;
        });
}
