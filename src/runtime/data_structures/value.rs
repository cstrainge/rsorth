
use std::{ cell::RefCell,
           fmt::{ self, Display, Formatter },
           hash::{ Hash, Hasher } };
use crate::{ lang::{ tokenizing::{ NumberType, Token },
                     code::{ ByteCode, pretty_print_code } },
             runtime::{ data_structures::{ byte_buffer::ByteBufferPtr,
                                           data_object::DataObjectPtr,
                                           value_hash::ValueHashPtr,
                                           value_vec::{ ValueVec, ValueVecPtr } },
                        error::{ self, script_error },
                        interpreter::Interpreter } };



/// Core value enumeration used by the Strange Forth interpreter.  This enumeration used to
/// represent all data types that the interpreter and the underlying Forth code can understand and
/// manipulate.
#[derive(Clone, PartialOrd)]
pub enum Value
{
    /// The value represents nothing and no data is associated.
    None,

    /// We have an integer value.  Represented as an i64.
    Int(i64),

    /// A floating-point value  Represented as a f64.
    Float(f64),

    /// A boolean value.
    Bool(bool),

    /// A string value, represented by a Rust string.
    String(String),

    /// A vector of Values.  Handed by reference with a ValueVecPtr.
    Vec(ValueVecPtr),

    /// A hash map of Values/Values.  Handled by reference with a ValueHashPtr.
    HashMap(ValueHashPtr),

    /// A Forth structure.  Handled by reference with a DataObjectPtr.
    DataObject(DataObjectPtr),

    /// A buffer for holding binary data.
    ByteBuffer(ByteBufferPtr),

    /// A Forth source code token.
    Token(Token),

    /// A block of interpreter byte-code.
    Code(ByteCode)
}



/// Convert an arbitrary data type to a Value.
pub trait ToValue
{
    /// Implement to handle the actual conversion.
    fn to_value(&self) -> Value;
}



/// Convert a borrowed string into a Value.
impl ToValue for &String
{
    fn to_value(&self) -> Value
    {
        let string = (*self).clone();
        Value::String(string)
    }
}


/// Allow code to create a default Value object.
impl Default for Value
{
    fn default() -> Value
    {
        Value::None
    }
}


/// Implement Eq for the Value enumeration.  This is used so that the Value enumeration can be used
/// as a key in a hash map.  However, it must be noted that the Value can hold floating point values
/// which violate the Eq trait rules.  This is a known limitation of the implementation.
///
/// It should be noted in the user documentation that floating point Values should not be used as
/// keys in a hash map.
impl Eq for Value {}



/// Manage equality for the Value enumeration.  This implements the various rules for value
/// conversion when comparing two Values.
impl PartialEq for Value
{
    fn eq(&self, other: &Value) -> bool
    {
        // If both are some kind of numbers attempt to manage the conversion.
        if Value::both_are_numeric(self, other)
        {
            if Value::either_is_float(self, other)
            {
                let a = self.get_float_val();
                let b = other.get_float_val();

                a == b
            }
            else if Value::either_is_int(self, other)
            {
                let a = self.get_int_val();
                let b = other.get_int_val();

                a == b
            }
            else if Value::either_is_bool(self, other)
            {
                let a = self.get_bool_val();
                let b = other.get_bool_val();

                a == b
            }
            else
            {
                false
            }
        }
        else if self.is_stringable() && other.is_stringable()
        {
            // It looks like it's possible to perform a simple string conversion, so compare the
            // values based on that.
            let a = self.get_string_val();
            let b = other.get_string_val();

            a == b
        }
        else
        {
            // Perform a direct comparison based on the other types, only actually attempting the
            // comparison if they are both of the same type.
            match ( self, other )
            {
                ( Value::Vec(a),        Value::Vec(b)        ) => *a.borrow() == *b.borrow(),
                ( Value::DataObject(a), Value::DataObject(b) ) => *a.borrow() == *b.borrow(),
                ( Value::Token(a),      Value::Token(b)      ) => a == b,
                ( Value::HashMap(a),    Value::HashMap(b)    ) => *a.borrow() == *b.borrow(),
                ( Value::ByteBuffer(a), Value::ByteBuffer(b) ) => *a.borrow() == *b.borrow(),
                ( Value::Code(a),       Value::Code(b)       ) => a == b,

                _                                              => false
            }
        }
    }
}


/// Compute the hash for a Value.  Falling back on the actual value type the Value represents.
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
            Value::HashMap(value)    => value.borrow().hash(state),
            Value::DataObject(value) => value.borrow().hash(state),
            Value::ByteBuffer(value) => value.borrow().hash(state),
            Value::Token(value)      => value.hash(state),
            Value::Code(value)       => value.hash(state)
        }
    }
}


/// Pretty print the value for display.
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
            Value::HashMap(value)    => write!(f, "{}", value.borrow()),
            Value::DataObject(value) => write!(f, "{}", value.borrow()),
            Value::ByteBuffer(value) => write!(f, "{}", value.borrow()),
            Value::Token(value)      => write!(f, "{}", value),
            Value::Code(value)       => write!(f, "{}", pretty_print_code(None, value))
        }
    }
}



/// Define implementations for converting between Values and the raw data types they represent.
macro_rules! value_conversion
{
    ($data_type:ty , $variant:ident , $as_ident:ident) =>
    {
        #[doc = concat!("Convert a value to ", stringify!($data_type), ".")]
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


        #[doc = concat!("Allow conversion from ", stringify!($data_type), " to a Value.")]
        impl ToValue for $data_type
        {
            fn to_value(&self) -> Value
            {
                Value::$variant(self.clone())
            }
        }


        #[doc = concat!("Support converting from a ", stringify!($data_type), " to a Value.")]
        impl From<$data_type> for Value
        {
            fn from(original: $data_type) -> Value
            {
                original.to_value()
            }
        }


        #[doc = concat!("Also support converting from a Value to a ", stringify!($data_type), ".")]
        impl From<Value> for $data_type
        {
            fn from(original: Value) -> $data_type
            {
                if let Value::$variant(contained_value) = original
                {
                    return contained_value;
                }

                panic!("Could not automatically convert from a Value to a {}.", stringify!($type));
            }
        }
    };
}



/// Hand implement ToValue for the NumberType token enumeration.
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



/// Convenience implementation for converting a usize to a Value.  The usize type is not represented
/// directly in the Value enumeration, so it is converted to an i64 internally.
impl ToValue for usize
{
    fn to_value(&self) -> Value
    {
        Value::Int(*self as i64)
    }
}



/// Convenience implementation for converting a u64 to a Value.  The u64 type is not represented
/// directly in the Value enumeration, so it is converted to an i64 internally.
impl ToValue for u64
{
    fn to_value(&self) -> Value
    {
        Value::Int(*self as i64)
    }
}



/// Used to convert a Vector of value compatible types to a ValueVec based Value.
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



/// Used to convert reference to a Vector of value compatible types to a ValueVec based Value.
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



// Implement the simple conversions for the value enumeration types.
value_conversion!(i64,           Int,        as_int);
value_conversion!(f64,           Float,      as_float);
value_conversion!(bool,          Bool,       as_bool);
value_conversion!(String,        String,     as_string);
value_conversion!(ValueVecPtr,   Vec,        as_vec);
value_conversion!(ValueHashPtr,  HashMap,    as_hash_map);
value_conversion!(DataObjectPtr, DataObject, as_data_object);
value_conversion!(ByteBufferPtr, ByteBuffer, as_byte_buffer);
value_conversion!(Token,         Token,      as_token);
value_conversion!(ByteCode,      Code,       as_code);



/// Handily implement variant checks for the types the Value enumeration supports.
macro_rules! is_variant
{
    ($name:ident , $either_name:ident , $variant:ident) =>
    {
        #[doc = concat!("Check if the value is the variant ", stringify!($variant), ".")]
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

        #[doc = concat!("Check if either of the two values are the variant ",
                        stringify!($variant),
                        ".")]
        pub fn $either_name(a: &Value, b: &Value) -> bool
        {
            a.$name() || b.$name()
        }
    };
}



impl Value
{
    /// Check if the value is the None variant.
    pub fn is_none(&self) -> bool
    {
        if let Value::None = self { true } else { false }
    }

    /// Check if either of the two values are the None variant.
    pub fn either_is_none(a: &Value, b: &Value) -> bool
    {
        a.is_none() || b.is_none()
    }

    // Create variant checks for the other supported types.
    is_variant!(is_int,         either_is_int,         Int);
    is_variant!(is_float,       either_is_float,       Float);
    is_variant!(is_bool,        either_is_bool,        Bool);
    is_variant!(is_string,      either_is_string,      String);
    is_variant!(is_vec,         either_is_vec,         Vec);
    is_variant!(is_hash_map,    either_is_hash_map,    HashMap);
    is_variant!(is_data_object, either_is_data_object, DataObject);
    is_variant!(is_byte_buffer, either_is_byte_buffer, ByteBuffer);
    is_variant!(is_token,       either_is_token,       Token);
    is_variant!(is_code,        either_is_code,        Code);


    /// Is the value any kind of numeric variant type?
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


    /// Are both values numeric types?
    pub fn both_are_numeric(a: &Value, b: &Value) -> bool
    {
        a.is_numeric() && b.is_numeric()
    }


    // Does the Value represent a simply stringable type?
    pub fn is_stringable(&self) -> bool
    {
        match self
        {
            Value::None                 => true,
            Value::Int(_)               => true,
            Value::Float(_)             => true,

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


    /// Get a string that represents the value performing simple conversion if possible.
    pub fn get_string_val(&self) -> String
    {
        match self
        {
            Value::None                     => String::new(),
            Value::Int(value)               => value.to_string(),
            Value::Float(value)             => value.to_string(),
            Value::String(value)            => value.clone(),
            Value::Token(token) =>
                match token
                {
                    Token::String(_, value) => value.clone(),
                    Token::Word(_, word)    => word.clone(),
                    _                       => panic!("Value is not convertible to string.")
                }
            _                               => panic!("Value is not convertible to string.")
        }
    }


    /// Convert the Value to a boolean value, performing simple tests if it's not directly a boolean
    /// value.
    pub fn get_bool_val(&self) -> bool
    {
        match self
        {
            Value::None          => false,
            Value::Int(value)    => *value != 0,
            Value::Float(value)  => *value != 0.0,
            Value::Bool(value)   => *value,
            Value::String(value) => !value.is_empty(),
            _                    => true
        }
    }


    /// Convert the value to an integer value.  Performing simple conversions if it's not directly
    /// an integer value.  Only applicable to types that satisfy the is_numeric() test.
    pub fn get_int_val(&self) -> i64
    {
        match self
        {
            Value::None                              => 0,
            Value::Int(value)                        => *value,
            Value::Float(value)                      => *value as i64,
            Value::Bool(value)                       => if *value { 1 } else { 0 },
            Value::Token(token) =>
                match token
                {
                    Token::Number(_, num_type) =>
                        match num_type
                        {
                            NumberType::Int(value)   => *value,
                            NumberType::Float(value) => *value as i64
                        }
                    _                                => panic!("Value is not convertible to int.")
                }
            _                                        => panic!("Value is not convertible to int.")
        }
    }

    /// Convert the value to an floating point value.  Performing simple conversions if it's not
    /// directly an floating point value.  Only applicable to types that satisfy the is_numeric()
    /// test.
    pub fn get_float_val(&self) -> f64
    {
        match self
        {
            Value::None                              => 0.0,
            Value::Int(value)                        => *value as f64,
            Value::Float(value)                      => *value,
            Value::Bool(value)                       => if *value { 1.0 } else { 0.0 },
            Value::Token(token) =>
                match token
                {
                    Token::Number(_, num_type) =>
                        match num_type
                        {
                            NumberType::Int(value)   => *value as f64,
                            NumberType::Float(value) => *value
                        }
                    _                                => panic!("Value is not convertible to float.")
                }
            _                                        => panic!("Value is not convertible to float.")
        }
    }
}



impl Value
{
    /// Convert a string to a string that could be used directly within source code.  For example,
    /// new lines are converted to the \n escape sequence, etc.  The string is also enclosed in
    /// double quotes.
    ///
    /// Mainly used for debug, stack, and structure printing.
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



/// Implement the deep clone trait for the value enumeration and any sub-types that are handled by
/// reference.  The normal clone() operation only clones the reference itself, not the data it
/// contains.
pub trait DeepClone
{
    fn deep_clone(&self) -> Value;
}



/// Implement the deep clone trait for the Value enumeration, defaulting to shallow copies for value
/// types and deep copies for reference types.
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
            Value::HashMap(value)    => value.deep_clone(),
            Value::DataObject(value) => value.deep_clone(),
            Value::ByteBuffer(value) => value.deep_clone(),
            Value::Token(value)      => Value::Token(value.clone()),
            Value::Code(value)       => Value::Code(value.clone())
        }
    }
}



thread_local!
{
    /// Keep track of the current indentation level for pretty printing more structured values.  For
    /// example the HashMap and DataObject types use this to format their output.
    ///
    /// This value is thread safe and can be used for pretty printing values in multiple independent
    /// threads.
    static VALUE_FORMAT_INDENT: RefCell<usize> = RefCell::new(0);
}



/// Get the current indentation level in spaces for pretty printing structured values.  See
/// VALUE_FORMAT_INDENT for more details.
pub fn value_format_indent() -> usize
{
    let mut indent: usize = 0;

    VALUE_FORMAT_INDENT.with(|value|
        {
            indent = *value.borrow();
        });

    indent
}


/// Increment the pretty printing indentation level for some values.  See VALUE_FORMAT_INDENT for
/// more details.
pub fn value_format_indent_inc()
{
    VALUE_FORMAT_INDENT.with(|value|
        {
            *value.borrow_mut() += 4;
        });
}


/// Decrement the pretty printing indentation level for some values.  See VALUE_FORMAT_INDENT for
/// more details.
pub fn value_format_indent_dec()
{
    VALUE_FORMAT_INDENT.with(|value|
        {
            *value.borrow_mut() -= 4;
        });
}
