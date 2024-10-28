
use std::{ collections::HashMap,
           fmt::{ self, Display, Formatter },
           ops::{ Index, IndexMut } };
use crate::runtime::data_structures::contextual_data::ContextualData;



/// The runtime of a word in the Strange Forth dictionary.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum WordRuntime
{
    /// The word is executed immediately when found at compile time in the user scrip.
    Immediate,

    /// The word is executed normally after the script has been fully compiled.
    Normal
}



/// The type of a word in the Strange Forth dictionary.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum WordType
{
    /// The word is a native word written in Rust.
    Native,

    /// The word is a script word written in the Strange Forth language.
    Scripted
}



/// The visibility of a word in the Strange Forth user listing.  Particularly useful when the user
/// executes `.w` within the Strange Forth repl.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum WordVisibility
{
    /// The word is visible within the directory listing.
    Visible,

    /// The word is hidden from the directory listing.
    Hidden
}



/// The information stored in the Strange Forth word dictionary for each word.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct WordInfo
{
    /// The name of the word.
    pub name: String,


    /// When should the word be executed?
    pub runtime: WordRuntime,

    /// What kind of word is it?
    pub word_type: WordType,

    /// Is the word visible in the directory listing?
    pub visibility: WordVisibility,


    /// A simple description of the word.
    pub description: String,

    /// The stack signature of the word.
    pub signature: String,


    /// The index of the actual handler for the word in the interpreter's handler list.
    pub handler_index: usize
}


impl WordInfo
{
    /// Create a new WordInfo with default values.
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



/// A sub dictionary of words is kept for each context in the main dictionary struct.
type SubDictionary = HashMap<String, WordInfo>;


/// The stack of contextual sub-dictionaries that make up the entire dictionary.
type DictionaryStack = Vec<SubDictionary>;



/// The Strange Forth dictionary used by the interpreter.  We use this to keep track of all of the
/// words defined within the interpreter.  This dictionary is contextual so words can be defined
/// within sub-contexts and forgotten when that context is released.
///
/// Primarily used by Forth words to manage and release their own variables and constants.
pub struct Dictionary
{
    stack: DictionaryStack
}



/// Implementation of the context management for the dictionary.
impl ContextualData for Dictionary
{
    /// Mark a new context.  Any words added to the dictionary after this point will be lost when
    /// the corresponding release_context is called.
    fn mark_context(&mut self)
    {
        self.stack.push(SubDictionary::new());
    }


    /// Release the current context and free all of the words within it.  This will panic if there
    /// are no contexts to release, or if we are trying to release the last context in the
    /// dictionary.
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


/// Allow the dictionary to be indexed by word names.
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


/// Allow the dictionary to be indexed by word names.
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


/// Pretty print the dictionary.  Words will appear only once in the listing.  So if a word is
/// overridden in a second context only the newest version of the word will be shown.
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
    /// Create a new empty dictionary with a default context.  This context will be the root context
    /// and should never be freed.
    pub fn new() -> Dictionary
    {
        let mut new_dictionary = Dictionary
            {
                stack: Vec::new()
            };

        new_dictionary.mark_context();

        new_dictionary
    }


    /// Insert a new word and it's info into the dictionary.  This word will be added into the top
    /// context.
    pub fn insert(&mut self, name: String, info: WordInfo)
    {
        let top = self.top_mut();
        let _ = top.insert(name, info);
    }


    /// Get a merged contextless version of the dictionary.  Words will appear only once in the
    /// listing.  So if a word is overridden in a second context only the newest version of the word
    /// will be shown.
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


    /// Try to get a word from the dictionary.  This will search all contexts within the dictionary
    /// returning only the newest version of the word if found.
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

    /// Try to get a word from the dictionary.  This will search all contexts within the dictionary
    /// returning only the newest version of the word if found.
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


    /// Internal use only.  Get the top context within the dictionary.
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
