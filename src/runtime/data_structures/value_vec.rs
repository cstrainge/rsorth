
use std::{ cell::RefCell,
           collections::VecDeque,
           fmt::{ self, Display, Formatter },
           hash::Hash,
           ops::{ Index, IndexMut },
           rc::Rc };
use crate::runtime::data_structures::value::Value;

use super::value::{DeepClone, ToValue};



/// A vector of interpreter values.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct ValueVec
{
    values: VecDeque<Value>
}



/// A pointer to the ValueVec used to manage this object by reference.
pub type ValueVecPtr = Rc<RefCell<ValueVec>>;
// TODO: Investigate: pub type ValueVecPtr = Arc<Mutex<ValueVec>>;



/// Pretty print the ValueVec for debugging and other uses.
impl Display for ValueVec
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "[ ")?;

        for ( index, value ) in self.values.iter().enumerate()
        {
            if value.is_string()
            {
                write!(f, "{}", Value::stringify(&value.get_string_val()))?;
            }
            else
            {
                write!(f, "{}", self.values[index])?;
            }

            if index < self.values.len() - 1
            {
                write!(f, ", ")?;
            }
            else
            {
                write!(f, " ")?;
            }
        }

        write!(f, "]")
    }
}


/// Make sure that it's possible to create a complexly separate copy of the ValueVec while keeping
/// the copy logically equivalent to the original.
impl DeepClone for ValueVec
{
    fn deep_clone(&self) -> Value
    {
        let new_values = self.values.iter().map(|value| value.deep_clone()).collect();
        let vec_ptr = Rc::new(RefCell::new(ValueVec { values: new_values }));

        vec_ptr.to_value()
    }
}


/// Make sure that it's possible to create a complexly separate copy of the ValueVec while keeping
/// the copy logically equivalent to the original.
impl DeepClone for ValueVecPtr
{
    fn deep_clone(&self) -> Value
    {
        self.borrow().deep_clone()
    }
}


/// Access a ValueVec value by index.
impl Index<usize> for ValueVec
{
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output
    {
        if index >= self.len()
        {
            panic!("Index {} out of bounds {}!", index, self.len());
        }

        &self.values[index]
    }
}


/// Access a mutable ValueVec value by index.
impl IndexMut<usize> for ValueVec
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        if index >= self.len()
        {
            panic!("Index {} out of bounds {}!", index, self.len());
        }

        &mut self.values[index]
    }
}


/// Core functionality for the ValueVec type.
impl ValueVec
{
    /// Create a new reference to a ValueVec with a given size.
    pub fn new(new_size: usize) -> ValueVecPtr
    {
        let values = VecDeque::from(vec![Value::default(); new_size]);
        Rc::new(RefCell::new(ValueVec { values }))
    }


    /// Create a new reference to a ValueVec with a given vector of values.
    pub fn from_vec(values: Vec<Value>) -> ValueVecPtr
    {
        let values = VecDeque::from(values);
        Rc::new(RefCell::new(ValueVec { values }))
    }


    /// Make sure users of the ValueVec can iterate it's values.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<Value>
    {
        self.values.iter()
    }


    /// How big is the ValueVec?
    pub fn len(&self) -> usize
    {
        self.values.len()
    }


    /// Grow or shrink the ValueVec filling with None values as needed.
    pub fn resize(&mut self, new_size: usize)
    {
        self.values.resize(new_size, Value::None);
    }


    /// Append a clone of a ValueVec onto the end of another ValueVec.
    pub fn extend(&mut self, other: &ValueVec)
    {
        self.values.extend(other.values.iter().map(|item| item.deep_clone()));
    }


    /// Insert a value into an arbitrary location within the ValueVec.
    pub fn insert(&mut self, index: usize, value: Value)
    {
        self.values.insert(index, value);
    }


    /// Remove a value from an arbitrary location from within the ValueVec.
    pub fn remove(&mut self, index: usize)
    {
        let _ = self.values.remove(index);
    }


    /// Push a new value onto the front of the ValueVec.
    pub fn push_front(&mut self, value: Value)
    {
        self.values.push_front(value);
    }


    /// Pop a value from the front of the ValueVec.
    pub fn pop_front(&mut self) -> Option<Value>
    {
        if self.values.is_empty()
        {
            return None;
        }

        self.values.pop_front()
    }


    /// Push a new value onto the back of the ValueVec.
    pub fn push_back(&mut self, value: Value)
    {
        self.values.push_back(value);
    }


    /// Pop a value from the back of the ValueVec.
    pub fn pop_back(&mut self) -> Option<Value>
    {
        self.values.pop_back()
    }

}
