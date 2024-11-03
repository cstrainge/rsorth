
use std::{ borrow::Cow,
           cell::RefCell,
           collections::HashMap,
           ffi::CStr,
           os::raw::{ c_char,
                      c_void },
           rc::Rc,
           sync::Arc };
use libffi::{ low::{ ffi_abi_FFI_DEFAULT_ABI,
                     ffi_cif,
                     ffi_type,
                     types },
              raw::{ ffi_call,
                     ffi_prep_cif,
                     ffi_status_FFI_OK } };
use libloading::{ Library,
                  Symbol };
use crate::{ add_native_word,
             runtime::{ data_structures::{ byte_buffer::{ Buffer,
                                                          ByteBuffer },
                                           dictionary::{ WordRuntime,
                                                         WordType,
                                                         WordVisibility },
                                           value::{ ToValue,
                                                    Value } },
                        error::{ self,
                                 script_error,
                                 script_error_str },
                        interpreter::Interpreter } };



/// The calculated size of a type and any extra space needed for referenced data.
type CalculatedSize = ( usize, usize );

/// Conversion function from a Value to a native type.
type ConversionFrom = Arc<dyn Fn(&mut dyn Interpreter,
                                &Value,
                                usize,
                                &mut dyn Buffer,
                                &mut dyn Buffer) -> error::Result<()> + Send + Sync>;

/// Conversion function from a native type to a Value.
type ConversionTo = Arc<dyn Fn(&mut dyn Interpreter,
                               usize,
                               &mut dyn Buffer) -> error::Result<Value> + Send + Sync>;

/// Calculate the size of a type and any extra space needed for referenced data.  For example, a
/// string type would take up the space for a pointer to the string data, and the string data itself
/// plus it's null terminator lives in the extra buffer.
type ConversionSize = Arc<dyn Fn(&mut dyn Interpreter,
                                 usize,
                                 &Value) -> error::Result<CalculatedSize> + Send + Sync>;

// Calculate the base size of a type without any extra space needed for referenced data.
type BaseSize = Arc<dyn Fn(usize) -> usize + Send + Sync>;



/// Structure that holds information about a type that can be used in the ffi interface, as well as
/// conversion handler functions for that type.
#[derive(Clone)]
struct TypeInfo
{
    ffi_type: *mut ffi_type,
    name: String,

    conversion_from: ConversionFrom,
    conversion_to: ConversionTo,
    conversion_size: ConversionSize,
    base_size: BaseSize
}



/// Structure that holds the ffi interface libraries and the types that can be used with those
/// libraries.
pub struct FfiInterface
{
    libs: HashMap<String, Arc<RefCell<Library>>>,
    types: HashMap<String, Arc<TypeInfo>>
}


impl FfiInterface
{
    /// Create a new default ffi interface with all the base types pre-registered.
    pub fn new() -> FfiInterface
    {
        FfiInterface
            {
                libs: HashMap::new(),
                types: FfiInterface::default_types()
            }
    }

    /// Reset the ffi interface to it's default state.
    pub fn reset(&mut self)
    {
        self.libs.clear();
        self.types = FfiInterface::default_types();
    }

    /// Create the default type information for the ffi interface.
    fn default_types() -> HashMap<String, Arc<TypeInfo>>
    {
        HashMap::from_iter(vec![
            (
                "ffi.void".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.void".to_string(),
                    ffi_type: &raw mut types::void,
                    conversion_from: Arc::new(|_interpreter, _value, _align, _buffer, _extra|
                    {
                        Ok(())
                    }),
                    conversion_to: Arc::new(|_interpreter, _align, _buffer|
                    {
                        Ok(Value::None)
                    }),
                    conversion_size: Arc::new(|_interpreter, _align, _value|
                    {
                        Ok(( 0, 0 ))
                    }),
                    base_size: Arc::new(|_align|
                    {
                        0
                    })
                })
            ),
            (
                "ffi.bool".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.bool".to_string(),
                    ffi_type: &raw mut types::uint8,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<bool>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<bool>(), false, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<bool>(), align);
                        Ok(( size_of::<bool>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<bool>(), align);
                        size_of::<bool>() + padding
                    })
                })
            ),
            (
                "ffi.i8".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.i8".to_string(),
                    ffi_type: &raw mut types::sint8,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<i8>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<i8>(), true, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<i8>(), align);
                        Ok(( size_of::<i8>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<i8>(), align);
                        size_of::<i8>() + padding
                    })
                })
            ),
            (
                "ffi.u8".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.u8".to_string(),
                    ffi_type: &raw mut types::uint8,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                        value,
                                                        align,
                                                        size_of::<u8>(),
                                                        buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<u8>(), false, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<u8>(), align);
                        Ok(( size_of::<u8>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<u8>(), align);
                        size_of::<u8>() + padding
                    })
                })
            ),
            (
                "ffi.i16".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.i16".to_string(),
                    ffi_type: &raw mut types::sint16,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                        value,
                                                        align,
                                                        size_of::<i16>(),
                                                        buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<i16>(), true, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<i16>(), align);
                        Ok(( size_of::<i16>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<i16>(), align);
                        size_of::<i16>() + padding
                    })
                })
            ),
            (
                "ffi.u16".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.u16".to_string(),
                    ffi_type: &raw mut types::uint16,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<u16>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<u16>(), false, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<u16>(), align);
                        Ok(( size_of::<u16>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<u16>(), align);
                        size_of::<u16>() + padding
                    })
                })
            ),
            (
                "ffi.i32".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.i32".to_string(),
                    ffi_type: &raw mut types::sint32,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                        value,
                                                        align,
                                                        size_of::<i32>(),
                                                        buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<i32>(), true, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<i32>(), align);
                        Ok(( size_of::<i32>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<i32>(), align);
                        size_of::<i32>() + padding
                    })
                })
            ),
            (
                "ffi.u32".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.u32".to_string(),
                    ffi_type: &raw mut types::uint32,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                        value,
                                                        align,
                                                        size_of::<u32>(),
                                                        buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<u32>(), false, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<u32>(), align);
                        Ok(( size_of::<u32>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<u32>(), align);
                        size_of::<u32>() + padding
                    })
                })
            ),
            (
                "ffi.i64".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.i64".to_string(),
                    ffi_type: &raw mut types::sint64,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                        value,
                                                        align,
                                                        size_of::<i64>(),
                                                        buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<i64>(), true, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<i64>(), align);
                        Ok(( size_of::<i64>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<i64>(), align);
                        size_of::<i64>() + padding
                    })
                })
            ),
            (
                "ffi.u64".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.u64".to_string(),
                    ffi_type: &raw mut types::uint64,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_int(interpreter,
                                                        value,
                                                        align,
                                                        size_of::<u64>(),
                                                        buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_int(align, size_of::<u64>(), false, buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<u64>(), align);
                        Ok(( size_of::<u64>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<u64>(), align);
                        size_of::<u64>() + padding
                    })
                })
            ),
            (
                "ffi.f32".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.f32".to_string(),
                    ffi_type: &raw mut types::float,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_float(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<f32>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_float(align, size_of::<f32>(), buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<f32>(), align);
                        Ok(( size_of::<f32>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<f32>(), align);
                        size_of::<f32>() + padding
                    })
                })
            ),
            (
                "ffi.f64".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.f64".to_string(),
                    ffi_type: &raw mut types::double,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, _extra|
                    {
                        FfiInterface::conversion_to_float(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<f64>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_from_float(align, size_of::<f64>(), buffer)
                    }),
                    conversion_size: Arc::new(|_interpreter, align, _value|
                    {
                        let padding = FfiInterface::alignment(size_of::<f64>(), align);
                        Ok(( size_of::<f64>() + padding, 0 ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(size_of::<f64>(), align);
                        size_of::<f64>() + padding
                    })
                })
            ),
            (
                "ffi.string".to_string(),
                Arc::new(TypeInfo
                {
                    name: "ffi.string".to_string(),
                    ffi_type: &raw mut types::pointer,
                    conversion_from: Arc::new(|interpreter, value, align, buffer, extra|
                    {
                        if !value.is_string()
                        {
                            return script_error_str(interpreter, "Value is not a string.");
                        }

                        let string = value.to_string();

                        let ptr_size = size_of::<*const c_void>();
                        let ptr_padding = FfiInterface::alignment(ptr_size, align);

                        let str_size = string.len();
                        let str_padding = FfiInterface::alignment(str_size, align);

                        buffer.write_int(ptr_size, extra.position_ptr_mut() as i64);
                        buffer.increment_position(ptr_padding);
                        extra.write_string(str_size + str_padding, &string);

                        Ok(())
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        let padding = FfiInterface::alignment(size_of::<*const c_char>(), align);
                        let raw_ptr = buffer.read_int(size_of::<*const c_char>(),
                                                      false) as *const c_char;

                        buffer.increment_position(padding);

                        let string = match unsafe { CStr::from_ptr(raw_ptr).to_string_lossy() }
                            {
                                Cow::Borrowed(string) => string.to_string(),
                                Cow::Owned(string) => string
                            };

                        buffer.increment_position(string.len() + 1);

                        Ok(string.to_value())
                    }),
                    conversion_size: Arc::new(|interpreter, align, value|
                    {
                        if !value.is_string()
                        {
                            return script_error_str(interpreter, "Value is not a string.");
                        }

                        let ptr_padding = FfiInterface::alignment(size_of::<*const c_void>(), align);
                        let string_len = value.get_string_val().len() + 1;

                        let string_padding = FfiInterface::alignment(string_len, align);

                        Ok(( size_of::<*const c_void>() + ptr_padding, string_len + string_padding ))
                    }),
                    base_size: Arc::new(|align|
                    {
                        let padding = FfiInterface::alignment(2, align);
                        size_of::<*const c_void>() + padding
                    })
                })
            )
        ])
    }

    /// Calculate the padding needed to align a value to the given alignment.
    fn alignment(size: usize, align: usize) -> usize
    {
        let aligned_size = (size + align - 1) & !(align - 1);
        let padding = aligned_size - size;

        padding
    }

    /// Convert a Value to a native integer type.
    fn conversion_to_int(interpreter: &mut dyn Interpreter,
                         value: &Value,
                         align: usize,
                         size: usize,
                         buffer: &mut dyn Buffer) -> error::Result<()>
    {
        let padding = FfiInterface::alignment(size, align);

        if !value.is_numeric()
        {
            return script_error_str(interpreter, "Value is not numeric.");
        }

        buffer.write_int(size, value.get_int_val());
        buffer.increment_position(padding);

        Ok(())
    }

    /// Convert from a native integer type to a integer Value.
    fn conversion_from_int(align: usize,
                           size: usize,
                           is_signed: bool,
                           buffer: &mut dyn Buffer) -> error::Result<Value>
    {
        let padding = FfiInterface::alignment(size, align);

        let value = buffer.read_int(size, is_signed);

        buffer.increment_position(padding);
        Ok(value.to_value())
    }

    /// Convert a Value to a native floating point type.
    fn conversion_to_float(interpreter: &mut dyn Interpreter,
                           value: &Value,
                           align: usize,
                           size: usize,
                           buffer: &mut dyn Buffer) -> error::Result<()>
    {
        let padding = FfiInterface::alignment(size, align);

        if !value.is_numeric()
        {
            return script_error_str(interpreter, "Value is not numeric.");
        }

        buffer.write_float(size, value.get_float_val());
        buffer.increment_position(padding);

        Ok(())
    }

    /// Convert from a native floating point type to a floating point Value.
    fn conversion_from_float(align: usize,
                             size: usize,
                             buffer: &mut dyn Buffer) -> error::Result<Value>
    {
        let padding = FfiInterface::alignment(size, align);

        let value = buffer.read_float(size);

        buffer.increment_position(padding);
        Ok(value.to_value())
    }
}


/// Structure that handles a word that calls a foreign function.
struct FfiWord
{
    /// The library that contains the function.
    library: Arc<RefCell<Library>>,


    /// The name of the library.
    library_name: String,

    /// The name of the function to call in the library.
    function_name: String,


    /// The types of the arguments to the function.
    arg_types: Vec<Arc<TypeInfo>>,

    /// The function's return type.
    return_type: Arc<TypeInfo>,


    /// The alignment of the function's arguments and return value.
    alignment: usize
}


/// Implement the Fn trait for FfiWord to make the struct callable.
impl Fn<( &mut dyn Interpreter, )> for FfiWord
{
    extern "rust-call" fn call(&self, args: ( &mut dyn Interpreter, ) ) -> error::Result<()>
    {
        self.handle_word(args.0)
    }
}


/// Implement the FnMut trait for FfiWord to make the struct callable.
impl FnMut<( &mut dyn Interpreter, )> for FfiWord
{
    extern "rust-call" fn call_mut(&mut self, args: ( &mut dyn Interpreter, )) -> error::Result<()>
    {
        self.handle_word(args.0)
    }
}


/// Implement the FnOnce trait for the FfiWord to make the struct callable.
impl FnOnce<( &mut dyn Interpreter, )> for FfiWord
{
    type Output = error::Result<()>;

    extern "rust-call" fn call_once(self, args: ( &mut dyn Interpreter, )) -> error::Result<()>
    {
        self.handle_word(args.0)
    }
}


impl FfiWord
{
    /// Create a new FfiWord handler.
    pub fn new(library: Arc<RefCell<Library>>,
               library_name: String,
               function_name: String,
               arg_types: Vec<Arc<TypeInfo>>,
               return_type: Arc<TypeInfo>) -> FfiWord
    {
        FfiWord
            {
                library,
                library_name,
                function_name,
                arg_types,
                return_type,
                alignment: 8
            }
    }

    /// Handle the word by calling the foreign function.
    fn handle_word(&self, interpreter: &mut dyn Interpreter) -> error::Result<()>
    {
        let library = self.library.borrow();

        // Get the function from the library.
        let function: Symbol<*mut c_void> =
            match unsafe { library.get(self.function_name.as_bytes()) }
            {
                Ok(function) => function,
                Err(error) => return script_error(interpreter,
                                                  format!("Failed to get library {} symbol {}: {}.",
                                                          self.library_name,
                                                          self.function_name,
                                                          error))
            };

        // Allocate the buffers for the parameters and populate them.  We also get the array of
        // pointers to the parameters.
        let mut buffer = ByteBuffer::new(0);
        let mut extra_buffer = ByteBuffer::new(0);

        let mut param_value_ptrs = self.get_param_value_ptrs(interpreter,
                                                             &mut buffer,
                                                             &mut extra_buffer)?;

        // Allocate the buffer for the return value.
        let mut return_buffer = ByteBuffer::new((self.return_type.base_size)(self.alignment));

        // Create the array of raw ffi_type pointers for creating the ffi_cif.
        let mut arg_types =
            {
                let mut arg_types = Vec::with_capacity(self.arg_types.len());

                for type_info in self.arg_types.iter()
                {
                    arg_types.push(type_info.as_ref().ffi_type);
                }

                arg_types
            };

        // Create teh ffi cif and if successful call the function.
        let mut cif: ffi_cif = unsafe { std::mem::zeroed() };
        let code_ptr = unsafe { Some(std::mem::transmute(*function)) };

        let status = unsafe
            {
                ffi_prep_cif(&mut cif,
                             ffi_abi_FFI_DEFAULT_ABI,
                             arg_types.len() as u32,
                             self.return_type.ffi_type,
                             arg_types.as_mut_ptr())
            };

        if status != ffi_status_FFI_OK
        {
            return script_error_str(interpreter, "Failed to create FFI cif.");
        }

        unsafe
        {
            ffi_call(&mut cif,
                     code_ptr,
                     return_buffer.byte_ptr_mut(),
                     param_value_ptrs.as_mut_ptr());
        }

        // Convert the return value to an interpreter Value and push it onto the data stack.  But
        // only if the return type is not void.
        let value = (self.return_type.conversion_to)(interpreter,
                                                     self.alignment,
                                                     &mut return_buffer)?;

        if !value.is_none()
        {
            interpreter.push(value);
        }

        // All done.
        Ok(())
    }

    /// Pop the parameters from the data stack, convert them to the native types in the supplied
    /// byte buffers, and return a vector of pointers to the converted values.
    fn get_param_value_ptrs(&self,
                            interpreter: &mut dyn Interpreter,
                            buffer: &mut ByteBuffer,
                            extra_buffer: &mut ByteBuffer) -> error::Result<Vec<*mut c_void>>
    {
        let args_len = self.arg_types.len();

        let mut arg_values: Vec<Value> = Vec::with_capacity(args_len);
        let mut arg_value_ptrs = Vec::with_capacity(args_len);

        let mut base_size = 0;
        let mut extra_size = 0;

        arg_values.resize(args_len, Value::None);

        for index in 0..args_len
        {
            let index = args_len - index - 1;
            let value = interpreter.pop()?;
            let ( size, extra ) = (self.arg_types[index].conversion_size)(interpreter,
                                                                          self.alignment,
                                                                          &value)?;

            base_size += size;
            extra_size += extra;

            arg_values[index] = value;
        }

        buffer.resize(base_size);
        extra_buffer.resize(extra_size);

        for index in 0..args_len
        {
            arg_value_ptrs.push(buffer.position_ptr_mut());
            (self.arg_types[index].conversion_from)(interpreter,
                                                    &arg_values[index],
                                                    self.alignment,
                                                    buffer,
                                                    extra_buffer)?;
        }

        Ok(arg_value_ptrs)
    }
}



/// Load a native library and register it with the ffi interface under the library's alias name.
fn word_ffi_open(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let register_name = interpreter.pop_as_string()?;
    let lib_name = interpreter.pop_as_string()?;

    if interpreter.ffi().libs.contains_key(&register_name)
    {
        error::script_error(interpreter, format!("Library {} is already loaded.", register_name))?;
    }

    let lib = unsafe { Library::new(lib_name.clone()) };

    match lib
    {
        Ok(lib) =>
            {
                let _ = interpreter.ffi_mut().libs.insert(register_name,
                                                          Arc::new(RefCell::new(lib)));
            },

        Err(error) =>
            {
                return script_error(interpreter, format!("Failed to load library {}: {}.",
                                                         lib_name,
                                                         error));
            }
    }

    Ok(())
}

/// Create a new word that calls a foreign function.
fn word_ffi_fn(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let ret_type_name = interpreter.pop_as_string()?;
    let param_type_names = interpreter.pop_as_array()?;
    let mut fn_alias = interpreter.pop_as_string()?;
    let fn_name = interpreter.pop()?;
    let lib_name = interpreter.pop_as_string()?;

    // Get the location of the name of the function from the script.
    let location = fn_name.as_token(interpreter)?.location();

    let fn_name = fn_name.get_string_val();

    // If the alias is empty, use the function name as the alias.
    if fn_alias.is_empty()
    {
        fn_alias = fn_name.clone();
    }

    // Get the library from the ffi interface.  Then check to see if the function is in the library.
    let lib: Arc<RefCell<Library>> = match interpreter.ffi().libs.get(&lib_name)
        {
            Some(lib) => lib.clone(),
            None => return script_error(interpreter, format!("Library {} is not loaded.", lib_name))
        };

    if let Err(error) = unsafe { lib.borrow().get::<Symbol<*mut c_void>>(fn_name.as_bytes()) }
    {
        return script_error(interpreter, format!("Failed to get symbol {} from library {}: {}.",
                                                 fn_name,
                                                 lib_name,
                                                 error));
    }

    // Get the type information for the parameter types.
    let arg_type_infos =
        {
            let mut arg_type_infos = Vec::with_capacity(param_type_names.borrow().len());

            for param_type_name in param_type_names.borrow().iter()
            {
                let param_type_name = match param_type_name.is_token()
                    {
                        true => param_type_name.to_string(),
                        false => return script_error_str(interpreter,
                                                       "Parameter type name, {}, is not a string.",
                                                       )
                    };

                let type_info = match interpreter.ffi().types.get(&param_type_name)
                    {
                        Some(type_info) => type_info,
                        None => return script_error(interpreter,
                                                    format!("Unknown ffi type name {}.",
                                                            param_type_name))
                    };

                arg_type_infos.push(type_info.clone());
            }

            arg_type_infos
        };

    // Get the type information for the return value.
    let ret_type_info = match interpreter.ffi().types.get(&ret_type_name)
        {
            Some(ret_type_info) => ret_type_info.clone(),
            None => return script_error(interpreter, format!("Unknown ffi type name {}.",
                                                             ret_type_name))
        };

    // Create the signature for the word's description.
    let arg_signature =
        {
            let mut signature = String::new();

            if arg_type_infos.len() > 0
            {
                for arg_type in arg_type_infos.iter()
                {
                    signature.push_str(&arg_type.name);
                    signature.push_str(" ");
                }

                signature.push_str("-- ");
            }
            else
            {
                signature = " -- ".to_string();
            }

            signature.push_str(&ret_type_name);

            signature
        };

    // Create the word handler for the foreign function, then add the new word to the interpreter.
    let word = FfiWord::new(lib, lib_name.clone(), fn_name.clone(), arg_type_infos, ret_type_info);

    interpreter.add_word(location.path().clone(),
                         location.line(),
                         location.column(),
                         fn_alias,
                         Rc::new(word),
                         format!("Call native function {} in library {}.", fn_name, lib_name),
                         arg_signature,
                         WordRuntime::Normal,
                         WordVisibility::Visible,
                         WordType::Native);

    Ok(())
}

// Create a new structure compatible with the ffi interface.
fn word_ffi_struct(_interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    Ok(())
}

// Register a new ffi array type for an existing ffi type.
fn word_ffi_array(_interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    Ok(())
}


// Register the ffi words with the interpreter.
pub fn register_ffi_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "ffi.load", word_ffi_open,
        "Load an binary library and register it with the ffi interface.",
        "lib-name -- ");

    add_native_word!(interpreter, "ffi.fn", word_ffi_fn,
        "Bind to an external function.",
        "lib-name fn-name fn-alias fn-params ret-name -- ");

    add_native_word!(interpreter, "ffi.#", word_ffi_struct,
        "Create a structure compatible with the ffi interface.",
        "found_initializers is_hidden types fields packing name [defaults] -- ");

    add_native_word!(interpreter, "ffi.[]", word_ffi_array,
        "Register a new ffi array type for the existing ffi type.",
        "struct-name -- ");
}
