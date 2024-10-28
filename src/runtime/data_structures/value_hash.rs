
use std::{ collections::HashMap,
           cell::RefCell,
           cmp::Ordering,
           fmt::{ self,
                  Display,
                  Formatter },
           hash::{ Hash,
                   Hasher },
           rc::Rc };
use crate::runtime::data_structures::value::{ DeepClone,
                                              ToValue,
                                              Value,
                                              value_format_indent_dec,
                                              value_format_indent_inc,
                                              value_format_indent };



/// A hash table used for storing relational data as needed by user scripts.  Both the keys and
/// values are Value types, allowing for a wide range of data types to be stored in the hash table.
/// Including other sub hash tables.
#[derive(Clone, Eq)]
pub struct ValueHash
{
    values: HashMap<Value, Value>
}


/// A reference counted pointer to a ValueHash.  This is the type that is managed by scripts.
pub type ValueHashPtr = Rc<RefCell<ValueHash>>;


/// Is one ValueHash logically equal to another ValueHash?  This can potentially be an expensive
/// operation.
impl PartialEq for ValueHash
{
    fn eq(&self, other: &ValueHash) -> bool
    {
        for ( key, value ) in &self.values
        {
            if !other.values.contains_key(key)
            {
                return false;
            }

            if other.values.get(key) != Some(value)
            {
                return false;
            }
        }

        true
    }
}


/// Useful for ordering operations.  This can potentially be an expensive operation.
impl PartialOrd for ValueHash
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        if self.values.len() != other.values.len()
        {
            return self.values.len().partial_cmp(&other.values.len());
        }

        let mut result = self.values.keys().partial_cmp(other.values.keys());

        if result == Some(Ordering::Equal)
        {
            result = self.values.values().partial_cmp(other.values.values());
        }

        result
    }
}


/// Allow the whole hash table to be hashed.  This can potentially be an expensive operation.
/// However it can allow HashTables to be used as keys for other Hash tables.
impl Hash for ValueHash
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        for ( key, value ) in &self.values
        {
            key.hash(state);
            value.hash(state);
        }
    }
}


/// Make sure we can create a completely separate copy of the hash table.
impl DeepClone for ValueHash
{
    fn deep_clone(&self) -> Value
    {
        let mut new_hash = ValueHash
            {
                values: HashMap::new()
            };

        for ( key, value ) in self.values.iter()
        {
            let new_key = key.deep_clone();
            let new_value = value.deep_clone();

            new_hash.values.insert(new_key, new_value);
        }

        Rc::new(RefCell::new(new_hash)).to_value()
    }
}


/// Make sure we can create a completely separate copy of hash table references.
impl DeepClone for ValueHashPtr
{
    fn deep_clone(&self) -> Value
    {
        self.borrow().deep_clone()
    }
}


/// Pretty print the hash table while maintaining it's logical structure.  Otherwise it could be
/// difficult to read.
impl Display for ValueHash
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{{\n")?;

        value_format_indent_inc();

        for ( index, ( key, value ) ) in self.values.iter().enumerate()
        {
            write!(f,
                   "{:width$}{} -> {} {}\n",
                   "",
                   if key.is_string()
                   {
                       Value::stringify(&key.get_string_val())
                   }
                   else
                   {
                       key.to_string()
                   },
                   if value.is_string()
                   {
                       Value::stringify(&value.get_string_val())
                   }
                   else
                   {
                       value.to_string()
                   },
                   if index < self.values.len() - 1 { "," } else { "" },
                   width = value_format_indent())?;
        }

        value_format_indent_dec();

        write!(f, "{:width$}}}", "", width = value_format_indent())
    }
}


/// Core implementation of the ValueHash type.
impl ValueHash
{
    /// Create a new and empty ValueHash reference.
    pub fn new() -> ValueHashPtr
    {
        let hash = ValueHash
            {
                values: HashMap::new()
            };

        Rc::new(RefCell::new(hash))
    }


    /// Get the size of the hash table.
    pub fn len(&self) -> usize
    {
        self.values.len()
    }


    /// Insert a key/value pair into the hash table, replacing the value if the key already exists.
    /// The kee is left unchanged in that case.
    pub fn insert(&mut self, key: Value, value: Value)
    {
        self.values.insert(key, value);
    }


    /// Try to get a value from the hash table by key.
    pub fn get(&self, key: &Value) -> Option<&Value>
    {
        self.values.get(key)
    }


    /// Grow a hash table by adding all the other tables values.  Replacing existing values with any
    /// overlapping keys.
    pub fn extend(&mut self, other: &ValueHash)
    {
        for ( key, value ) in other.values.iter()
        {
            self.values.insert(key.deep_clone(), value.deep_clone());
        }
    }


    /// Allow user code to iterate over the hash table.
    pub fn iter(&self) -> std::collections::hash_map::Iter<Value, Value>
    {
        self.values.iter()
    }
}
