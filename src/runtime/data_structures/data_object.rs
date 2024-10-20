
use std::{ fmt::{ self, Display, Formatter },
           rc::Rc,
           cell::RefCell,
           hash::{ Hash, Hasher } };
use crate::runtime::data_structures::value::{ Value,
                                              value_format_indent,
                                              value_format_indent_inc,
                                              value_format_indent_dec };



pub struct DataObjectDefinition
{
    name: String,
    field_names: Vec<String>,
    defaults: Vec<Value>
}



pub type DataObjectDefinitionPtr = Rc<RefCell<DataObjectDefinition>>;



#[derive(Clone)]
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
            fields[index] = definition.defaults[index].clone();
        }

        let data_object = DataObject
            {
                definition_ptr: definition_ptr.clone(),
                fields
            };

        Rc::new(RefCell::new(data_object))
     }
}
