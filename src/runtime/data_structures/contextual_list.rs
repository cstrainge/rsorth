
use std::ops::{ Index, IndexMut };
use crate::runtime::data_structures::contextual_data::ContextualData;



struct SubList<T>
{
    pub items: Vec<T>,
    pub start_index: usize
}


impl<T> SubList<T>
{
    fn new(start_index: usize) -> SubList<T>
    {
        SubList {
                items: Vec::new(),
                start_index
            }
    }
}



pub struct ContextualList<T>
   where
       T: Clone
{
    list_stack: Vec<SubList<T>>
}


impl<T> ContextualData for ContextualList<T>
    where
        T: Clone
{
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


impl<T> Index<usize> for ContextualList<T>
    where
        T: Clone
{
    type Output =  T;

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


impl<T> IndexMut<usize> for ContextualList<T>
    where
        T: Clone
{
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
    pub fn new() -> ContextualList<T>
    {
        let mut new_list = ContextualList
            {
                list_stack: Vec::new()
            };

        new_list.mark_context();

        new_list
    }

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

    pub fn insert(&mut self, value: &T) -> usize
    {
        let top = self.top_mut();

        top.items.push(value.clone());
        self.len() - 1
    }

    fn top(&self) -> &SubList<T>
    {
        if self.list_stack.is_empty()
        {
            panic!("Reading from an empty context!");
        }

        let index = self.list_stack.len() - 1;
        &self.list_stack[index]
    }

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
