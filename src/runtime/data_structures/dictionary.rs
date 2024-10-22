
use std::{ collections::HashMap,
           fmt::{ self, Display, Formatter },
           ops::{ Index, IndexMut } };
use crate::runtime::data_structures::contextual_data::ContextualData;



#[derive(Clone)]
pub enum WordRuntime
{
    Immediate,
    Normal
}


#[derive(Clone)]
pub enum WordType
{
    Native,
    Scripted
}


#[derive(Clone)]
pub enum WordVisibility
{
    Visible,
    Hidden
}


#[derive(Clone)]
pub struct WordInfo
{
    pub name: String,
    pub runtime: WordRuntime,
    pub word_type: WordType,
    pub visibility: WordVisibility,

    pub description: String,
    pub signature: String,

    pub handler_index: usize
}


impl WordInfo
{
    pub fn new() -> WordInfo
    {
        WordInfo
            {
                name: String::new(),
                runtime: WordRuntime::Normal,
                word_type: WordType::Native,
                visibility: WordVisibility::Visible,
                description: String::new(),
                signature: String::new(),
                handler_index: 0
            }
    }
}



type SubDictionary = HashMap<String, WordInfo>;

type DictionaryStack = Vec<SubDictionary>;



pub struct Dictionary
{
    stack: DictionaryStack
}


impl ContextualData for Dictionary
{
    fn mark_context(&mut self)
    {
        self.stack.push(SubDictionary::new());
    }

    fn release_context(&mut self)
    {
        if self.stack.is_empty()
        {
            panic!("Releasing an empty context!");
        }

        if self.stack.len() == 1
        {
            panic!("Releasing last context!");
        }

        let _ = self.stack.pop();
    }
}


impl Index<&String> for Dictionary
{
    type Output = WordInfo;

    fn index(&self, name: &String) -> &Self::Output
    {
        if let Some(found) = self.try_get(name)
        {
            return found;
        }

        panic!("Word {} not found in dictionary!", name);
    }
}


impl IndexMut<&String> for Dictionary
{
    fn index_mut(&mut self, name: &String) -> &mut Self::Output
    {
        if let Some(found) = self.try_get_mut(name)
        {
            return found;
        }

        panic!("Word {} not found in dictionary!", name);
    }
}


impl Display for Dictionary
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result
    {
        let merged = self.get_merged();
        let mut max_size = 0;
        let mut visible_words = 0;

        for item in merged.iter()
        {
            let size = item.0.len();

            if max_size < size
            {
                max_size = size;
            }

            if let WordVisibility::Visible = item.1.visibility
            {
                visible_words += 1;
            }
        }

        let mut string_result = format!("{} words defined.\n\n", visible_words);

        let mut keys: Vec<&String> = merged.keys().collect();
        keys.sort();

        for key in keys.iter()
        {
            let word = &merged[*key];

            if let WordVisibility::Visible = word.visibility
            {
                string_result = string_result +
                                &format!("{:width$}  {:6}",
                                         key,
                                         word.handler_index,
                                         width = max_size);

                string_result = string_result +
                    {
                        if let WordRuntime::Immediate = word.runtime
                        {
                            "  immediate"
                        }
                        else
                        {
                            "           "
                        }
                    };

                string_result = string_result + &format!("  --  {}\n", word.description);
            }
        }

        write!(formatter, "{}", string_result)
    }
}


impl Dictionary
{
    pub fn new() -> Dictionary
    {
        let mut new_dictionary = Dictionary
            {
                stack: Vec::new()
            };

        new_dictionary.mark_context();

        new_dictionary
    }

    pub fn insert(&mut self, name: String, info: WordInfo)
    {
        let top = self.top_mut();
        let _ = top.insert(name, info);
    }

    pub fn get_merged(&self) -> SubDictionary
    {
        let mut merged = SubDictionary::new();

        for sub_dictionary in self.stack.iter()
        {
            for (name, info) in sub_dictionary.iter()
            {
                let _ = merged.insert(name.clone(), info.clone());
            }
        }

        merged
    }

    pub fn try_get(&self, name: &String) -> Option<&WordInfo>
    {
        for sub_dictionary in self.stack.iter().rev()
        {
            if let Some(found) = sub_dictionary.get(name)
            {
                return Some(found);
            }
        }

        None
    }

    pub fn try_get_mut(&mut self, name: &String) -> Option<&mut WordInfo>
    {
        for sub_dictionary in self.stack.iter_mut().rev()
        {
            if let Some(found) = sub_dictionary.get_mut(name)
            {
                return Some(found);
            }
        }

        None
    }

    fn top(&self) -> &SubDictionary
    {
        if self.stack.is_empty()
        {
            panic!("Reading from an empty context!");
        }

        let index = self.stack.len() - 1;
        &self.stack[index]
    }

    fn top_mut(&mut self) -> &mut SubDictionary
    {
        if self.stack.is_empty()
        {
            panic!("Reading from an empty context!");
        }

        let index = self.stack.len() - 1;
        &mut self.stack[index]
    }
}
