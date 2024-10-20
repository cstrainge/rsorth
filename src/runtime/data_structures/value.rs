
use std::{ cell::RefCell,
           fmt::{ self, Display, Formatter },
           hash::{ Hash, Hasher } };
use crate::{ lang::tokenizing::NumberType,
             runtime::{ data_structures::data_object::DataObjectPtr,
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
    DataObject(DataObjectPtr)
}



pub trait ToValue
{
    fn to_value(&self) -> Value;
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
    fn eq(&self, _other: &Value) -> bool
    {
        false
    }
}


impl Hash for Value
{
    fn hash<H: Hasher>(&self, _state: &mut H)
    {
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
            Value::DataObject(value) => write!(f, "{}", value.borrow())
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
                                      &format!("Value could not be converted to {}",
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



value_conversion!(i64, Int, as_int);
value_conversion!(f64, Float, as_float);
value_conversion!(bool, Bool, as_bool);
value_conversion!(String, String, as_string);
value_conversion!(DataObjectPtr, DataObject, as_data_object);



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
                '\n' => result.push_str("\\\n"),
                '\r' => result.push_str("\\\r"),
                '\t' => result.push_str("\\\t"),
                '\\' => result.push_str("\\\\"),
                _    => result.push(character)
            }
        }

        result
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
