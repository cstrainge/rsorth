
use std::{ cell::RefCell,
           collections::VecDeque,
           fmt::{ self, Display, Formatter },
           hash::Hash,
           ops::{ Index, IndexMut },
           rc::Rc };
use crate::runtime::data_structures::value::Value;

use super::value::{DeepClone, ToValue};



#[derive(Clone, PartialEq, Hash)]
pub struct ValueVec
{
    values: VecDeque<Value>
}



pub type ValueVecPtr = Rc<RefCell<ValueVec>>;
// TODO: Investigate: pub type ValueVecPtr = Arc<Mutex<ValueVec>>;



impl Display for ValueVec
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "[ ")?;

        for ( index, value ) in self.values.iter().enumerate()
        {
            if value.is_string()
            {
                write!(f, "{}", Value::stringify(&value.get_string_val().unwrap()))?;
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


impl DeepClone for ValueVec
{
    fn deep_clone(&self) -> Value
    {
        let new_values = self.values.iter().map(|value| value.deep_clone()).collect();
        let vec_ptr = Rc::new(RefCell::new(ValueVec { values: new_values }));

        vec_ptr.to_value()
    }
}


impl DeepClone for ValueVecPtr
{
    fn deep_clone(&self) -> Value
    {
        self.borrow().deep_clone()
    }
}


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


impl ValueVec
{
    pub fn new(new_size: usize) -> ValueVecPtr
    {
        let values = VecDeque::from(vec![Value::default(); new_size]);
        Rc::new(RefCell::new(ValueVec { values }))
    }

    pub fn from_vec(values: Vec<Value>) -> ValueVecPtr
    {
        let values = VecDeque::from(values);
        Rc::new(RefCell::new(ValueVec { values }))
    }

    pub fn len(&self) -> usize
    {
        self.values.len()
    }

    pub fn resize(&mut self, new_size: usize)
    {
        self.values.resize(new_size, Value::default());
    }

    pub fn insert(&mut self, index: usize, value: Value)
    {
        self.values.insert(index, value);
    }

    pub fn remove(&mut self, index: usize)
    {
        let _ = self.values.remove(index);
    }

    pub fn push_front(&mut self, value: Value)
    {
        self.values.push_front(value);
    }

    pub fn pop_front(&mut self) -> Option<Value>
    {
        if self.values.is_empty()
        {
            return None;
        }

        self.values.pop_front()
    }

    pub fn push_back(&mut self, value: Value)
    {
        self.values.push_back(value);
    }

    pub fn pop_back(&mut self) -> Option<Value>
    {
        self.values.pop_back()
    }

}
