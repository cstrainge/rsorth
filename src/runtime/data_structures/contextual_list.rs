
use std::ops::{ Index, IndexMut };
use crate::runtime::data_structures::contextual_data::ContextualData;



/// Internal use only.  Represents a sub context within the entire list.
struct SubList<T>
{
    pub items: Vec<T>,
    pub start_index: usize
}


impl<T> SubList<T>
{
    /// Create a new sub-list starting at the given index.
    fn new(start_index: usize) -> SubList<T>
    {
        SubList {
                items: Vec::new(),
                start_index
            }
    }
}



/// A list that manages contexts of sub-lists.
pub struct ContextualList<T>
   where
       T: Clone
{
    list_stack: Vec<SubList<T>>
}


/// Manage marking and releasing of the list's contexts.
impl<T> ContextualData for ContextualList<T>
    where
        T: Clone
{
    /// Allocate a new list sub-context.
    fn mark_context(&mut self)
    {
        let start_index = if !self.list_stack.is_empty()
            {
                let top = &self.top();
                top.start_index + top.items.len()
            }
            else
            {
                0
            };

        self.list_stack.push(SubList::new(start_index));
    }

    /// Release the current context and all the data within it.  This will panic if the last context
    /// is released.
    fn release_context(&mut self)
    {
        if self.list_stack.is_empty()
        {
            panic!("Releasing an empty context!");
        }
        else if self.list_stack.len() == 1
        {
            panic!("Releasing last context!");
        }

        let _ = self.list_stack.pop();
    }
}


/// Allow for indexing within the list.
impl<T> Index<usize> for ContextualList<T>
    where
        T: Clone
{
    type Output =  T;

    /// Index into the list, regardless of the current context.  This will panic if the index is out
    /// of bounds of the entire list.
    fn index(&self, index: usize) -> &Self::Output
    {
        if index >= self.len()
        {
            panic!("Index {} out of bounds {}!", index, self.len());
        }

        for stack_item in self.list_stack.iter().rev()
        {
            if index >= stack_item.start_index
            {
                let index = index - stack_item.start_index;
                return &stack_item.items[index];
            }
        }

        panic!("Index {} not found.", index);
    }
}


/// Allow for indexing within a mutable list.
impl<T> IndexMut<usize> for ContextualList<T>
    where
        T: Clone
{
    /// Index into the list, regardless of the current context.  This will panic if the index is out
    /// of bounds of the entire list.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        if index >= self.len()
        {
            panic!("Index {} out of bounds {}!", index, self.len());
        }

        for stack_item in self.list_stack.iter_mut().rev()
        {
            if index >= stack_item.start_index
            {
                let index = index - stack_item.start_index;
                return &mut stack_item.items[index];
            }
        }

        panic!("Index {} not found.", index);
    }
}


impl<T> ContextualList<T>
    where
        T: Clone
{
    /// Create a new contextual list.  The new list starts empty with a default context.  This
    /// context should never be freed.  That is, there should always be at least one context managed
    /// by the list.
    pub fn new() -> ContextualList<T>
    {
        let mut new_list = ContextualList
            {
                list_stack: Vec::new()
            };

        new_list.mark_context();

        new_list
    }


    /// Get the length of the entire list, regardless of the current context.
    pub fn len(&self) -> usize
    {
        if !self.list_stack.is_empty()
        {
            let top = self.top();
            top.start_index + top.items.len()
        }
        else
        {
            0
        }
    }


    /// Insert a new value into the end of the list, returning the item's new index.  This will
    /// panic if there are no contexts in the list.
    pub fn insert(&mut self, value: T) -> usize
    {
        let top = self.top_mut();

        top.items.push(value);
        self.len() - 1
    }


    /// Internal use only, get the top context of the list.
    fn top(&self) -> &SubList<T>
    {
        if self.list_stack.is_empty()
        {
            panic!("Reading from an empty context!");
        }

        let index = self.list_stack.len() - 1;
        &self.list_stack[index]
    }


    /// Internal use only, get a mutable reference to the top context of the list.
    fn top_mut(&mut self) -> &mut SubList<T>
    {
        if self.list_stack.is_empty()
        {
            panic!("Reading mutably from an empty context!");
        }

        let index = self.list_stack.len() - 1;
        &mut self.list_stack[index]
    }
}
