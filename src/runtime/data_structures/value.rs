
use std::{ cell::RefCell,
           fmt::{ self, Display, Formatter },
           hash::{ Hash, Hasher } };
use crate::runtime::data_structures::data_object::DataObjectPtr;



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
