
use std::{ cell::RefCell,
           collections::HashMap,
           os::raw::c_void,
           rc::Rc,
           sync::Arc };
use libffi::{ low::*,
              raw::{ ffi_call,
                     ffi_prep_cif,
                     ffi_status_FFI_OK } };
use libloading::{ Library,
                  Symbol };
use crate::{ add_native_word,
             runtime::{ data_structures::{ byte_buffer::{ Buffer,
                                                          ByteBuffer },
                                           value::{ ToValue,
                                                    Value } },
                        error::{ self,
                                 script_error,
                                 script_error_str },
                        interpreter::Interpreter } };



type CalculatedSize = ( usize, usize );

type ConversionFrom = Arc<dyn Fn(&mut dyn Interpreter,
                                &Value,
                                usize,
                                &mut dyn Buffer,
                                &mut dyn Buffer) -> error::Result<()> + Send + Sync>;

type ConversionTo = Arc<dyn Fn(&mut dyn Interpreter,
                               usize,
                               &mut dyn Buffer) -> error::Result<Value> + Send + Sync>;

type ConversionSize = Arc<dyn Fn(&mut dyn Interpreter,
                                 usize,
                                 &Value) -> error::Result<CalculatedSize> + Send + Sync>;

type BaseSize = Arc<dyn Fn(usize) -> usize + Send + Sync>;



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



pub struct FfiInterface
{
    libs: HashMap<String, Arc<RefCell<Library>>>,
    types: HashMap<String, Arc<TypeInfo>>
}


impl FfiInterface
{
    pub fn new() -> FfiInterface
    {
        FfiInterface
            {
                libs: HashMap::new(),
                types: FfiInterface::default_types()
            }
    }

    pub fn reset(&mut self)
    {
        self.libs.clear();
        self.types = FfiInterface::default_types();
    }

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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<bool>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<bool>(), false, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<i8>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<i8>(), true, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<u8>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<u8>(), false, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<i16>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<i16>(), true, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<u16>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<u16>(), false, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<i32>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<i32>(), true, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<u32>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<u32>(), false, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<i64>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<i64>(), true, buffer)
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
                        FfiInterface::conversion_from_int(interpreter,
                                                          value,
                                                          align,
                                                          size_of::<u64>(),
                                                          buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_int(align, size_of::<u64>(), false, buffer)
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
                        FfiInterface::conversion_from_float(interpreter,
                                                            value,
                                                            align,
                                                            size_of::<f32>(),
                                                            buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_float(align, size_of::<f32>(), buffer)
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
                        FfiInterface::conversion_from_float(interpreter,
                                                            value,
                                                            align,
                                                            size_of::<f64>(),
                                                            buffer)
                    }),
                    conversion_to: Arc::new(|_interpreter, align, buffer|
                    {
                        FfiInterface::conversion_to_float(align, size_of::<f64>(), buffer)
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
                        FfiInterface::conversion_to_float(align, size_of::<*const c_void>(), buffer)
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

    fn alignment(size: usize, align: usize) -> usize
    {
        let aligned_size = (size + align - 1) & !(align - 1);
        let padding = aligned_size - size;

        padding
    }

    fn conversion_from_int(interpreter: &mut dyn Interpreter,
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

    fn conversion_to_int(align: usize,
                         size: usize,
                         is_signed: bool,
                         buffer: &mut dyn Buffer) -> error::Result<Value>
    {
        let padding = FfiInterface::alignment(size, align);

        let value = buffer.read_int(size, is_signed);

        buffer.increment_position(padding);
        Ok(value.to_value())
    }

    fn conversion_from_float(interpreter: &mut dyn Interpreter,
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

    fn conversion_to_float(align: usize,
                           size: usize,
                           buffer: &mut dyn Buffer) -> error::Result<Value>
    {
        let padding = FfiInterface::alignment(size, align);

        let value = buffer.read_float(size);

        buffer.increment_position(padding);
        Ok(value.to_value())
    }
}



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

fn word_ffi_fn(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let ret_type_name = interpreter.pop_as_string()?;
    let param_type_names = interpreter.pop_as_array()?;
    let mut fn_alias = interpreter.pop_as_string()?;
    let fn_name = interpreter.pop_as_string()?;
    let lib_name = interpreter.pop_as_string()?;

    if fn_alias.is_empty()
    {
        fn_alias = fn_name.clone();
    }

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

    let arg_type_infos: Rc<RefCell<Vec<Arc<TypeInfo>>>> =
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

            Rc::new(RefCell::new(arg_type_infos))
        };

    let arg_types =
        {
            let mut arg_types = Vec::with_capacity(arg_type_infos.borrow().len());

            for type_info in arg_type_infos.borrow().iter()
            {
                arg_types.push(type_info.as_ref().ffi_type);
            }

            Rc::new(RefCell::new(arg_types))
        };

    let ret_type_info = match interpreter.ffi().types.get(&ret_type_name)
        {
            Some(ret_type_info) => ret_type_info.clone(),
            None => return script_error(interpreter, format!("Unknown ffi type name {}.",
                                                             ret_type_name))
        };

    let arg_signature =
        {
            let mut signature = String::new();

            if arg_type_infos.borrow().len() > 0
            {
                for arg_type in arg_type_infos.borrow().iter()
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

    let fn_name_copy = fn_name.clone();
    let lib_name_copy = lib_name.clone();

    let word_handler = move |interpreter: &mut dyn Interpreter| -> error::Result<()>
        {
            let lib = match interpreter.ffi().libs.get(&lib_name)
                 {
                     Some(lib) => lib.clone(),
                     None => return script_error(interpreter,
                                                 format!("Library {} is not loaded.", lib_name))
                 };

            let lib = lib.borrow();
            let func: Symbol<*mut c_void> = match unsafe { lib.get(fn_name.as_bytes()) }
                {
                    Ok(func) => func,
                    Err(error) => return script_error(interpreter,
                                                      format!("Failed to get symbol {}: {}.",
                                                              fn_name,
                                                              error))
                };

            let args_len = arg_types.borrow().len();
            let mut cif: ffi_cif = unsafe { std::mem::zeroed() };

            let status = unsafe
                {
                    ffi_prep_cif(&mut cif,
                                 ffi_abi_FFI_DEFAULT_ABI,
                                 args_len as u32,
                                 ret_type_info.ffi_type,
                                 arg_types.borrow_mut().as_mut_ptr())
                };

            if status != ffi_status_FFI_OK
            {
                return script_error_str(interpreter, "Failed to create FFI cif.");
            }

            let mut buffer = ByteBuffer::new(0);
            let mut extra_buffer = ByteBuffer::new(0);

            let mut arg_value_ptrs: Vec<*mut c_void> =
                {
                    let mut arg_values: Vec<Value> = Vec::with_capacity(args_len);
                    let mut arg_value_ptrs = Vec::with_capacity(args_len);

                    arg_values.resize(args_len, Value::None);

                    let mut base_size = 0;
                    let mut extra_size = 0;

                    for index in 0..args_len
                    {
                        let index = args_len - index - 1;
                        let value = interpreter.pop()?;

                        let ( size, extra ) =
                                   (arg_type_infos.borrow()[index].conversion_size)(interpreter,
                                                                                    8,
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
                        (arg_type_infos.borrow()[index].conversion_from)(interpreter,
                                                                         &arg_values[index],
                                                                         8,
                                                                         &mut buffer,
                                                                         &mut extra_buffer)?;
                    }

                    arg_value_ptrs
                };

            let mut return_buffer = ByteBuffer::new((ret_type_info.base_size)(8));
            let code_ptr = unsafe { Some(std::mem::transmute(*func)) };

            unsafe
            {
                ffi_call(&mut cif,
                         code_ptr,
                         return_buffer.byte_ptr_mut(),
                         arg_value_ptrs.as_mut_ptr());
            }

            let value = (ret_type_info.conversion_to)(interpreter, 8, &mut return_buffer)?;
            interpreter.push(value);

            Ok(())
        };

    add_native_word!(interpreter,
                     fn_alias,
                     word_handler,
                     format!("Call native function {} in library {}.",
                             fn_name_copy,
                             lib_name_copy),
                     arg_signature);

    Ok(())
}

fn word_ffi_struct(_interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    Ok(())
}

fn word_ffi_array(_interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    Ok(())
}



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
