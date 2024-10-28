
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




#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct DataObjectDefinition
{
    name: String,
    field_names: Vec<String>,
    defaults: Vec<Value>,
    visibility: WordVisibility
}



pub type DataObjectDefinitionPtr = Rc<RefCell<DataObjectDefinition>>;

pub type DataDefinitionList = ContextualList<DataObjectDefinitionPtr>;




impl DataObjectDefinition
{
    pub fn new(name: String,
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

        Rc::new(RefCell::new(definition))
    }

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
            let field_index_accessor = Rc::new(move |interpreter: &mut dyn Interpreter| -> error::Result<()>
                {
                    interpreter.push(index.to_value());
                    Ok(())
                });

            let field_writer = Rc::new(move |interpreter: &mut dyn Interpreter| -> error::Result<()>
                {
                    let data_ptr = interpreter.pop_as_data_object()?;
                    let value = interpreter.pop()?;

                    data_ptr.borrow_mut().fields[index] = value;
                    Ok(())
                });

            let field_reader = Rc::new(move |interpreter: &mut dyn Interpreter| -> error::Result<()>
                {
                    let data_ptr = interpreter.pop_as_data_object()?;

                    interpreter.push(data_ptr.borrow().fields[index].clone());
                    Ok(())
                });

            let var_field_writer = Rc::new(move |interpreter: &mut dyn Interpreter|
                                                                                -> error::Result<()>
                {
                    let var_index = interpreter.pop_as_usize()?;
                    let value = interpreter.pop()?;
                    let data_ptr = interpreter.variables()[var_index].as_data_object(interpreter)?;

                    validate_index(interpreter, &var_index)?;

                    data_ptr.borrow_mut().fields[index] = value;
                    Ok(())
                });

            let var_field_reader = Rc::new(move |interpreter: &mut dyn Interpreter|
                                                                                -> error::Result<()>
                {
                    let var_index = interpreter.pop_as_usize()?;
                    let data_ptr = interpreter.variables()[var_index]
                                              .as_data_object(interpreter)?
                                              .clone();

                    validate_index(interpreter, &var_index)?;

                    interpreter.push(data_ptr.borrow().fields[index].clone());
                    Ok(())
                });

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

    pub fn name(&self) -> &String
    {
        &self.name
    }

    pub fn field_names(&self) -> &Vec<String>
    {
        &self.field_names
    }

    pub fn defaults(&self) -> &Vec<Value>
    {
        &self.defaults
    }

    pub fn visibility(&self) -> &WordVisibility
    {
        &self.visibility
    }
}



#[derive(Clone, Eq)]
pub struct DataObject
{
    pub definition_ptr: DataObjectDefinitionPtr,
    pub fields: Vec<Value>
}



pub type DataObjectPtr = Rc<RefCell<DataObject>>;



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


impl DeepClone for DataObjectPtr
{
    fn deep_clone(&self) -> Value
    {
        self.borrow().deep_clone()
    }
}


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
                   self.fields[index],
                   if index < self.fields.len() - 1 { "," } else { "" },
                   width = value_format_indent())?;
        }

        value_format_indent_dec();

        write!(f, "{:width$};", "", width = value_format_indent())
    }
}


impl DataObject
{
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
