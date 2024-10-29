
use std::{ cmp::Ordering,
           fmt::{ self, Display, Formatter },
           rc::Rc,
           cell::RefCell,
           hash::{ Hash, Hasher } };
use crate::{ lang::source_buffer::SourceLocation,
             runtime::{ error::{ self, script_error },
                        data_structures::{ contextual_list::ContextualList,
                                           dictionary::{ WordRuntime,
                                                         WordType,
                                                         WordVisibility },
                                           value::{ value_format_indent,
                                                    value_format_indent_dec,
                                                    value_format_indent_inc,
                                                    DeepClone,
                                                    ToValue,
                                                    Value } },
                      interpreter::Interpreter } };




/// The definition of a structured data object within a Strange Forth script.  This is used to
/// define the fields and hold the default value initializers for a structured data object.
///
/// The structure is readonly once created and it's fields are accessed by helper methods.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct DataObjectDefinition
{
    name: String,
    field_names: Vec<String>,
    defaults: Vec<Value>,
    visibility: WordVisibility
}



/// The interpreter manages these data objects by reference.
pub type DataObjectDefinitionPtr = Rc<RefCell<DataObjectDefinition>>;


/// Used to hold a list of data object definitions.
pub type DataDefinitionList = ContextualList<DataObjectDefinitionPtr>;



/// Display for the DataObjectDefinition structure.
impl Display for DataObjectDefinition
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "# {}", self.name)?;

        for field in &self.field_names
        {
            write!(f, " {}", field)?;
        }

        write!(f, " ;")
    }
}


impl DataObjectDefinition
{
    /// Create a new DataObjectDefinition reference.
    pub fn new(interpreter: &mut dyn Interpreter,
               name: String,
               field_names: Vec<String>,
               defaults: Vec<Value>,
               is_hidden: bool) -> DataObjectDefinitionPtr
    {
        let definition =
            DataObjectDefinition
            {
                name,
                field_names,
                defaults,
                visibility: if is_hidden { WordVisibility::Hidden } else { WordVisibility::Visible }
            };

        let definition_ptr = Rc::new(RefCell::new(definition));

        interpreter.add_structure_definition(definition_ptr.clone());

        definition_ptr
    }


    /// What is the name of the structure type?
    pub fn name(&self) -> &String
    {
        &self.name
    }


    /// List of field names for the structure type.
    pub fn field_names(&self) -> &Vec<String>
    {
        &self.field_names
    }


    /// List of the default values defined for the structure.
    // TODO: Change to initialization byte-code that will be executed on structure creation.
    pub fn defaults(&self) -> &Vec<Value>
    {
        &self.defaults
    }


    /// Should this structure and it's helper words be hidden from the user directory?
    pub fn visibility(&self) -> &WordVisibility
    {
        &self.visibility
    }


    /// Create the data access words for the given data object definition.
    ///
    /// For example, if the definition is for a structure named "Person" with fields "name", and
    /// "age..."  The following words will be created to create and access that structure's fields:
    ///
    ///    - Person.new
    ///    - Person.name
    ///    - Person.name!
    ///    - Person.name!!
    ///    - Person.name@
    ///    - Person.name@@
    ///    - Person.age
    ///    - Person.age!
    ///    - Person.age!!
    ///    - Person.age@
    ///    - Person.age@@
    ///
    /// The "new" word creates a new instance of the structure.
    ///
    /// The "Person.name*" and "Person.age*" words accesses the fields of the structure with varying
    /// degrees of convince.
    ///
    /// # Examples
    ///
    /// The word "Person.name" only pushes the index of the field onto the stack.  For example:
    ///
    /// ```
    /// ( Write a new name to a person variable. )
    /// "Bob" person @ Person.name #!
    /// ```
    ///
    /// Where as "Person.name!" combines the index and the write into a single operation:
    ///
    /// ```
    /// ( Write a new name to a person variable. )
    /// "Bob" person @ Person.name!
    /// ```
    ///
    /// Finally "Person.name!!" will also perform the variable dereference for you:
    ///
    /// ```
    /// ( Write a new name to a person variable.)
    /// "Bob" person Person.name!!
    /// ```
    pub fn create_data_definition_words(interpreter: &mut dyn Interpreter,
                                        location: Option<SourceLocation>,
                                        definition_ptr: DataObjectDefinitionPtr,
                                        is_hidden: bool)
    {
        let ( path, line, column ) =
            {
                if let Some(location) = location
                {
                    ( location.path().clone(), location.line(), location.column() )
                }
                else
                {
                    ( file!().to_string(), line!() as usize, column!() as usize )
                }
            };

        let struct_name = definition_ptr.borrow().name.clone();
        let visibility = if is_hidden { WordVisibility::Hidden } else { WordVisibility::Visible };

        let given_definition = definition_ptr.clone();

        // Register the structure creation word.
        interpreter.add_word(path.clone(),
                             line.clone(),
                             column.clone(),
                             format!("{}.new", struct_name),
                             Rc::new(move |interpreter: &mut dyn Interpreter| -> error::Result<()>
                             {
                                 let new_struct = DataObject::new(&given_definition);

                                 interpreter.push(new_struct.to_value());
                                 Ok(())
                             }),
                             format!("Create a new instance of the structure {}.", struct_name),
                             format!(" -- {}", struct_name),
                             WordRuntime::Normal,
                             visibility.clone(),
                             WordType::Native);

        // Helper function to validate the index of a variable.
        fn validate_index(interpreter: &dyn Interpreter,
                          var_index: &usize) -> error::Result<()>
        {
            if *var_index >= interpreter.variables().len()
            {
                script_error(interpreter,
                             format!("Index {} out of range for variable list {}.",
                                     var_index,
                                     interpreter.variables().len()))?;
            }

            Ok(())
        }

        for ( index, field_name ) in definition_ptr.borrow().field_names.iter().enumerate()
        {
            // Push the field index onto the stack.
            let field_index_accessor = Rc::new(move |interpreter: &mut dyn Interpreter| -> error::Result<()>
                {
                    interpreter.push(index.to_value());
                    Ok(())
                });

            // Write to a field of a structure found on the stack.
            let field_writer = Rc::new(move |interpreter: &mut dyn Interpreter| -> error::Result<()>
                {
                    let data_ptr = interpreter.pop_as_data_object()?;
                    let value = interpreter.pop()?;

                    data_ptr.borrow_mut().fields[index] = value;
                    Ok(())
                });

            // Read from a field from a structure found on the stack.
            let field_reader = Rc::new(move |interpreter: &mut dyn Interpreter| -> error::Result<()>
                {
                    let data_ptr = interpreter.pop_as_data_object()?;

                    interpreter.push(data_ptr.borrow().fields[index].clone());
                    Ok(())
                });

            // Write to a field of a structure variable found on the stack.
            let var_field_writer = Rc::new(move |interpreter: &mut dyn Interpreter|
                                                                                -> error::Result<()>
                {
                    let var_index = interpreter.pop_as_usize()?;
                    let value = interpreter.pop()?;

                    validate_index(interpreter, &var_index)?;
                    let data_ptr = interpreter.variables()[var_index].as_data_object(interpreter)?;

                    data_ptr.borrow_mut().fields[index] = value;
                    Ok(())
                });

            // Read from a field from a structure variable found on the stack.
            let var_field_reader = Rc::new(move |interpreter: &mut dyn Interpreter|
                                                                                -> error::Result<()>
                {
                    let var_index = interpreter.pop_as_usize()?;

                    validate_index(interpreter, &var_index)?;
                    let data_ptr = interpreter.variables()[var_index]
                                              .as_data_object(interpreter)?
                                              .clone();

                    interpreter.push(data_ptr.borrow().fields[index].clone());
                    Ok(())
                });

            // Register all of these structure field access words.
            interpreter.add_word(path.clone(),
                                 line.clone(),
                                 column.clone(),
                                 format!("{}.{}", struct_name, field_name),
                                 field_index_accessor,
                                 format!(""),
                                 format!(" -- {}-index", field_name),
                                 WordRuntime::Normal,
                                 visibility.clone(),
                                 WordType::Native);

            interpreter.add_word(path.clone(),
                                 line.clone(),
                                 column.clone(),
                                 format!("{}.{}!", struct_name, field_name),
                                 field_writer,
                                 format!("Write to the structure {} field {}.",
                                         struct_name,
                                         field_name),
                                 "value struct -- ".to_string(),
                                 WordRuntime::Normal,
                                 visibility.clone(),
                                 WordType::Native);

            interpreter.add_word(path.clone(),
                                 line.clone(),
                                 column.clone(),
                                 format!("{}.{}@", struct_name, field_name),
                                 field_reader,
                                 format!("Read from the structure {} field {}.",
                                         struct_name,
                                         field_name),
                                 "struct -- value".to_string(),
                                 WordRuntime::Normal,
                                 visibility.clone(),
                                 WordType::Native);

            interpreter.add_word(path.clone(),
                                 line.clone(),
                                 column.clone(),
                                 format!("{}.{}!!", struct_name, field_name),
                                 var_field_writer,
                                 format!("Write to the structure variable {} field {}.",
                                         struct_name,
                                         field_name),
                                 "value struct-var -- ".to_string(),
                                 WordRuntime::Normal,
                                 visibility.clone(),
                                 WordType::Native);

            interpreter.add_word(path.clone(),
                                 line.clone(),
                                 column.clone(),
                                 format!("{}.{}@@", struct_name, field_name),
                                 var_field_reader,
                                 format!("Read from the structure variable {} field {}.",
                                         struct_name,
                                         field_name),
                                 "struct-ver -- value".to_string(),
                                 WordRuntime::Normal,
                                 visibility.clone(),
                                 WordType::Native);
        }
    }
}



/// The actual data object instance used directly by user scripts.  Contains a reference to it's
/// definition and a list of fields for reading and writing.
#[derive(Clone, Eq)]
pub struct DataObject
{
    pub definition_ptr: DataObjectDefinitionPtr,
    pub fields: Vec<Value>
}



/// Reference to the data object instance used by the interpreter.
pub type DataObjectPtr = Rc<RefCell<DataObject>>;



/// Allow for the comparison of two data objects.
impl PartialEq for DataObject
{
    fn eq(&self, other: &DataObject) -> bool
    {
        if !(self.definition_ptr.borrow().name == other.definition_ptr.borrow().name)
        {
            return false;
        }

        for index in 0..self.fields.len()
        {
            if !(self.fields[index] == other.fields[index])
            {
                return false;
            }
        }

        true
    }
}


/// Allow for the ordering of two data objects.
impl PartialOrd for DataObject
{
    fn partial_cmp(&self, other: &DataObject) -> Option<Ordering>
    {
        let self_name = &self.definition_ptr.borrow().name;
        let other_name = &other.definition_ptr.borrow().name;

        if self_name != other_name
        {
            return self_name.partial_cmp(other_name);
        }

        self.fields.partial_cmp(&other.fields)
    }
}


/// Allow for the hashing of a data object.
impl Hash for DataObject
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        for field in &self.fields
        {
            field.hash(state);
        }
    }
}


/// Make sure a data object can be properly deep cloned.
impl DeepClone for DataObject
{
    fn deep_clone(&self) -> Value
    {
        let fields = self.fields.iter().map(|value| value.deep_clone()).collect();
        let data_object = DataObject
            {
                definition_ptr: self.definition_ptr.clone(),
                fields
            };

        Rc::new(RefCell::new(data_object)).to_value()
    }
}



/// Make sure a data object can be properly deep cloned.
impl DeepClone for DataObjectPtr
{
    fn deep_clone(&self) -> Value
    {
        self.borrow().deep_clone()
    }
}


/// Pretty print a data object while preserving the structure's shape for better readability.
impl Display for DataObject
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "# {}\n", self.definition_ptr.borrow().name)?;

        value_format_indent_inc();

        for index in 0..self.fields.len()
        {
            write!(f,
                   "{:width$}{} -> {} {}\n",
                   "",
                   self.definition_ptr.borrow().field_names[index],
                   if self.fields[index].is_string()
                   {
                       Value::stringify(&self.fields[index].get_string_val())
                   }
                   else
                   {
                       self.fields[index].to_string()
                   },
                   if index < self.fields.len() - 1 { "," } else { "" },
                   width = value_format_indent())?;
        }

        value_format_indent_dec();

        write!(f, "{:width$};", "", width = value_format_indent())
    }
}


impl DataObject
{
    /// Crate a new data object based on it's base definition.
    pub fn new(definition_ptr: &DataObjectDefinitionPtr) -> DataObjectPtr
    {
       let definition = definition_ptr.borrow();
       let mut fields = Vec::new();

       fields.resize(definition.defaults.len(), Value::default());

       for index in 0..fields.len()
       {
           fields[index] = definition.defaults[index].deep_clone();

       }

       let data_object = DataObject
           {
               definition_ptr: definition_ptr.clone(),
               fields
           };

       Rc::new(RefCell::new(data_object))
    }
}
