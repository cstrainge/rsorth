
use std::{ fs::{ metadata, canonicalize },
           path::{ Path, PathBuf },
           rc::Rc };
use crate::{ add_native_word,
             lang::{ code::{ /*pretty_print_code,*/ ByteCode, Op },
                     compilation::{ process_source_from_tokens,
                                    CodeConstructor,
                                    CodeConstructorList },
                     source_buffer::SourceLocation,
                     tokenizing::{ tokenize_from_file,
                                   tokenize_from_source,
                                   NumberType,
                                   Token,
                                   TokenList } },
             runtime::{ data_structures::{ contextual_data::ContextualData,
                                           contextual_list::ContextualList,
                                           data_object::{ DataDefinitionList,
                                                          DataObjectPtr },
                                           dictionary::{ Dictionary,
                                                         WordInfo,
                                                         WordRuntime,
                                                         WordType,
                                                         WordVisibility },
                                           value::{ DeepClone,
                                                    ToValue,
                                                    Value },
                                            value_hash::ValueHashPtr,
                                            value_vec::ValueVecPtr },
                        error::{ self, script_error, script_error_str },
                        interpreter::{ CallItem,
                                       CallStack,
                                       CodeManagement,
                                       Interpreter,
                                       InterpreterStack,
                                       ThreadManagement,
                                       ValueStack,
                                       VariableList,
                                       WordHandler,
                                       WordHandlerInfo,
                                       WordManagement } } };



pub type SearchPaths = Vec<String>;

pub type WordList = ContextualList<WordHandlerInfo>;



pub struct SorthInterpreter
{
    max_depth: usize,

    search_paths: SearchPaths,

    stack: ValueStack,

    current_location: Option<SourceLocation>,
    call_stack: CallStack,

    data_definitions: DataDefinitionList,

    dictionary: Dictionary,
    word_handlers: WordList,

    variables: VariableList,

    constructors: CodeConstructorList
}


impl Interpreter for SorthInterpreter
{
    fn add_search_path(&mut self, path: &String) -> error::Result<()>
    {
        if let Err(err) = metadata(path)
        {
            script_error(self, format!("Could not append search path {}: {}.", path, err))?;
        }

        self.search_paths.push(path.clone());
        Ok(())
    }

    fn add_search_path_for_file(&mut self, file_path: &String) -> error::Result<()>
    {
        let canonical = canonicalize(file_path)?;
        let path = Path::new(&canonical);

        if let Some(directory) = path.parent()
        {
            if directory.exists()
            {
                if let Some(directory) = directory.to_str()
                {
                    self.add_search_path(&directory.to_string())
                }
                else
                {
                    script_error_str(self, "Path contains error characters.")
                }
            }
            else
            {
                script_error(self, format!("Path {} does not exist.", directory.display()))
            }
        }
        else
        {
            script_error(self, format!("Could not get parent directory for file {}.", file_path))
        }
    }

    fn drop_search_path(&mut self) -> error::Result<()>
    {
        if self.search_paths.is_empty()
        {
            script_error_str(self, "Search path stack underflow.")?;
        }

        let _ = self.search_paths.pop();
        Ok(())
    }

    fn search_paths(&self) -> &Vec<String>
    {
        &self.search_paths
    }

    fn find_file(&self, path: &String) -> error::Result<String>
    {
        if Path::new(path).exists()
        {
            let canonical = canonicalize(path)?;

            if let Some(canonical) = canonical.to_str()
            {
                return Ok(canonical.to_string());
            }
            else
            {
                return script_error_str(self, "Path contains invalid characters.");
            }
        }
        else
        {
            for directory in self.search_paths.iter().rev()
            {
                let full_path = PathBuf::from(directory).join(path);

                if full_path.exists()
                {
                    let canonical = canonicalize(full_path)?;

                    if let Some(canonical) = canonical.to_str()
                    {
                        return Ok(canonical.to_string())
                    }
                    else
                    {
                        return script_error_str(self, "Path contains invalid characters.")?;
                    }
                }
            }

            script_error(self, format!("File {} not found.", path))
        }
    }

    fn variables(&self) -> &VariableList
    {
        &self.variables
    }

    fn dictionary(&self) -> &Dictionary
    {
        &self.dictionary
    }

    fn reset(&mut self) -> error::Result<()>
    {
        self.release_context();
        self.stack.clear();

        Ok(())
    }
}


impl ContextualData for SorthInterpreter
{
    fn mark_context(&mut self)
    {
        self.dictionary.mark_context();
        self.word_handlers.mark_context();
        self.data_definitions.mark_context();
        self.variables.mark_context();
    }

    fn release_context(&mut self)
    {
        self.dictionary.release_context();
        self.word_handlers.release_context();
        self.data_definitions.release_context();
        self.variables.release_context();
    }
}


impl InterpreterStack for SorthInterpreter
{
    fn stack_max_depth(&self) -> usize
    {
        self.max_depth
    }

    fn stack(&self) -> &ValueStack
    {
        &self.stack
    }

    fn push(&mut self, value: Value)
    {
        self.stack.push(value);

        if self.stack.len() > self.max_depth
        {
            self.max_depth = self.stack.len();
        }
    }

    fn pop(&mut self) -> error::Result<Value>
    {
        let item = self.stack.pop();

        if let None = item
        {
            script_error_str(self, "Stack underflow.")?;
        }

        Ok(item.unwrap())
    }

    fn pop_as_int(&mut self) -> error::Result<i64>
    {
        let value = self.pop()?;

        if !value.is_numeric()
        {
            script_error_str(self, "Expected numeric value.")?;
        }

        Ok(value.get_int_val())
    }

    fn pop_as_usize(&mut self) -> error::Result<usize>
    {
        Ok(self.pop_as_int()? as usize)
    }

    fn pop_as_float(&mut self) -> error::Result<f64>
    {
        let value = self.pop()?;

        if !value.is_numeric()
        {
            script_error_str(self, "Expected numeric value.")?;
        }

        Ok(value.get_float_val())
    }

    fn pop_as_bool(&mut self) -> error::Result<bool>
    {
        let value = self.pop()?;

        if !value.is_numeric()
        {
            script_error_str(self, "Expected boolean value.")?;
        }

        Ok(value.get_bool_val())
    }

    fn pop_as_string(&mut self) -> error::Result<String>
    {
        let value = self.pop()?;

        if !value.is_stringable()
        {
            script_error_str(self, "Expected a string value.")?;
        }

        Ok(value.get_string_val())
    }

    fn pop_as_array(&mut self) -> error::Result<ValueVecPtr>
    {
        let value = self.pop()?;

        if !value.is_vec()
        {
            script_error_str(self, "Expected an array.")?;
        }

        Ok(value.as_vec(self)?.clone())
    }

    fn pop_as_hash_map(&mut self) -> error::Result<ValueHashPtr>
    {
        let value = self.pop()?;

        if !value.is_hash_map()
        {
            script_error_str(self, "Expected a hash map.")?;
        }

        Ok(value.as_hash_map(self)?.clone())
    }

    fn pop_as_data_object(&mut self) -> error::Result<DataObjectPtr>
    {
        let value = self.pop()?;

        if !value.is_data_object()
        {
            script_error_str(self, "Expected a data object.")?;
        }

        Ok(value.as_data_object(self)?.clone())
    }

    fn pop_as_code(&mut self) -> error::Result<ByteCode>
    {
        let value = self.pop()?;

        if !value.is_code()
        {
            script_error_str(self, "Expected a code block.")?;
        }

        Ok(value.as_code(self)?.clone())
    }

    fn pick(&mut self, index: usize) -> error::Result<Value>
    {
        let value = self.stack.remove(index);
        return Ok(value);
    }

    fn push_to(&mut self, index: usize) -> error::Result<()>
    {
        if let Some(value) = self.stack.pop()
        {
            self.stack.insert(index, value);
        }
        else
        {
            script_error_str(self, "Stack underflow.")?;
        }

        Ok(())
    }

}


// Helper methods for the interpreter instruction handling.
impl SorthInterpreter
{
    fn define_variable(&mut self, value: &Value) -> error::Result<()>
    {
        if !value.is_stringable()
        {
            script_error(self, format!("Invalid variable name {}.", value))?;
        }
        else
        {
            // Get the name, and an index that will represent the variable.
            let name = value.get_string_val();
            let index = self.variables.insert(Value::default());

            // Create a new handler that will access the variable by index.
            let handler = move |interpreter: &mut dyn Interpreter|
            {
                interpreter.push(index.to_value());
                Ok(())
            };

            add_native_word!(self,
                             name,
                             handler,
                             format!("Access the index for variable {}.", name),
                             " -- variable_index");
        }

        Ok(())
    }

    fn define_constant(&mut self, value: &Value) -> error::Result<()>
    {
        if !value.is_stringable()
        {
            script_error(self, format!("Invalid constant name {}.", value))?;
        }
        else
        {
            // Get the name, and the new constant value.
            let name = value.get_string_val();
            let constant = self.pop()?;

            // Create a new handler that will push the constant value onto the stack.
            let handler = move |interpreter: &mut dyn Interpreter|
            {
                interpreter.push(constant.deep_clone());
                Ok(())
            };

            add_native_word!(self,
                             name,
                             handler,
                             &format!("Access value for constant {}.", name),
                             " -- constant_value");
        }

        Ok(())
    }

    fn read_variable(&mut self) -> error::Result<()>
    {
        // Make sure we have a valid index of the correct type.
        let index = self.pop_as_int()?;
        let value =
            {
                if (index as usize) >= self.variables.len()
                {
                    script_error(self, format!("Read index {} out of range of variable set.",
                                               index))?;
                }

                self.variables[index as usize].clone()
            };

        // Perform the read.
        self.push(value);
        Ok(())
    }

    fn write_variable(&mut self) -> error::Result<()>
    {
        // Make sure we have a valid index of the correct type.
        let index = self.pop_as_int()?;
        let value = self.pop()?;

        if (index as usize) >= self.variables.len()
        {
            script_error(self, format!("Write index {} out of range of variable set.", index))?;
        }

        // Perform the write.
        self.variables[index as usize] = value;
        Ok(())
    }

    fn execute_value(&mut self, value: &Value) -> error::Result<()>
    {
        let location = self.current_location.clone();

        // Execute the value based on it's type.  It can either be a string name, or an index to the
        // handler.  Any other value type is invalid.
        match value
        {
            Value::String(word_name) =>
                 {
                    self.execute_word_named(&location, word_name)
                 },

            Value::Token(token) =>
                {
                    match token
                    {
                        Token::Word(location, word_name) =>
                            {
                                self.execute_word_named(&Some(location.clone()), word_name)
                            },

                        _ =>
                            {
                                script_error(self,
                                             format!("Token {} is not executable.", token))
                            }
                    }
                },

            Value::Int(index) =>
                {
                    self.execute_word_index(&location, *index as usize)
                },

            _ =>
                {
                    script_error(self, format!("Value {} is not executable.", value))
                }
        }
    }

    fn push_constant_value(&mut self, value: &Value) -> error::Result<()>
    {
        // Make sure we don't push a reference to the original constant value.
        let new_value = value.deep_clone();

        self.push(new_value);
        Ok(())
    }

    fn absolute_index(&self, pc: usize, relative_index: &Value) -> error::Result<usize>
    {
        // Compute an absolute index from the relative index encoded within the original
        // instruction.
        let absolute =
            if relative_index.is_numeric()
            {
                (pc as i64 + relative_index.get_int_val()) as usize
            }
            else
            {
                script_error(self, format!("Invalid loop exit index {}.",
                                           relative_index))?;
                0
            };

        // All's good.
        Ok(absolute)
    }

    fn jump_if_match(&mut self,
                     pc: &mut usize,
                     relative_index: &Value,
                     test_value: bool) -> error::Result<()>
    {
        // Grab the test value from the stack and compute the absolute index from the instruction.
        // We pop from the stack first because we don't want the stack to be unbalanced even if
        // we get errors.
        let found_value = self.pop_as_bool()?;
        let absolute = self.absolute_index(*pc, relative_index)?;

        // Do we have a match?
        if found_value == test_value
        {
            // Account for the increment that still happens at the end of the execution loop.
            *pc = absolute - 1;
        }

        Ok(())
    }
}


impl CodeManagement for SorthInterpreter
{
    fn next_token(&mut self) -> error::Result<Token>
    {
        let optional_token = self.context_mut().next_token();

        match optional_token
        {
            Some(token) => Ok(token),
            None        => script_error_str(self, "Reading past end of token stream.")
        }
    }

    fn next_token_text(&mut self) -> error::Result<String>
    {
        let text = self.next_token()?.text(self)?.clone();
        Ok(text)
    }

    fn next_token_string(&mut self) -> error::Result<String>
    {
        let string = self.next_token()?.string(self)?.clone();
        Ok(string)
    }

    fn next_token_number(&mut self) -> error::Result<NumberType>
    {
        let number = self.next_token()?.number(self)?.clone();
        Ok(number)
    }

    fn next_token_word(&mut self) -> error::Result<( SourceLocation, String )>
    {
        let token = self.next_token()?;
        let word = token.word(self)?.clone();
        let location = token.location().clone();

        Ok((location, word ))
    }

    fn context_new(&mut self, tokens: TokenList)
    {
        self.constructors.push(CodeConstructor::new(tokens));
    }

    fn context_drop(&mut self) -> error::Result<()>
    {
        if self.constructors.is_empty()
        {
            script_error_str(self, "Compile context stack underflow.")?;
        }

        let _ = self.constructors.pop();
        Ok(())
    }

    fn context(&self) -> &CodeConstructor
    {
        if self.constructors.is_empty()
        {
            panic!("No compile context available.");
        }

        self.constructors.last().unwrap()
    }

    fn context_mut(&mut self) -> &mut CodeConstructor
    {
        if self.constructors.is_empty()
        {
            panic!("No compile context available.");
        }

        self.constructors.last_mut().unwrap()
    }

    fn process_source_file(&mut self, path: &String) -> error::Result<()>
    {
        let full_path = self.find_file(path)?;
        let tokens = tokenize_from_file(&full_path)?;

        self.add_search_path_for_file(&full_path)?;

        let result = process_source_from_tokens(tokens, self);

        self.drop_search_path()?;

        result
    }

    fn process_source(&mut self, path: &String, source: &String) -> error::Result<()>
    {
        let tokens = tokenize_from_source(path, source)?;
        process_source_from_tokens(tokens, self)
    }

    fn execute_code(&mut self, name: &String, code: &ByteCode) -> error::Result<()>
    {
        // Keep track of any contexts that get marked so that we can safely clean up if any releases
        // are missed.
        let mut contexts: usize = 0;

        fn cleanup_contexts(interpreter: &mut dyn Interpreter,
                            contexts: usize,
                            report_error: bool) -> error::Result<()>
        {
            for _ in 0..contexts
            {
                interpreter.release_context();
            }

            if report_error && contexts > 0
            {
                script_error_str(interpreter, "Unbalanced context handling detected.")?;
            }

            Ok(())
        }

        // Keep track of whether the call stack was pushed so that we can properly clean up after.
        let mut call_stack_pushed = false;

        // Keep track of any loops that are executed and their start/end points.
        let mut loop_locations = Vec::<( usize, usize )>::new();

        // Keep track of any try/catch blocks.
        let mut catch_locations = Vec::<usize>::new();

        // Now, we can execute the code.
        let mut pc = 0;

        while pc < code.len()
        {
            // Fetch the current instruction.
            let instruction = &code[pc];

            // Does the current instruction have a location associated with it?
            if let Some(location) = &instruction.location
            {
                self.current_location = Some(location.clone());
                self.call_stack_push(name.clone(), location.clone());
                call_stack_pushed = true;
            }

            // Keep track of wether the instruction was successful.
            let result: error::Result<()> =
                match &instruction.op
                {
                    Op::DefVariable(value)       => self.define_variable(value),

                    Op::DefConstant(value)       => self.define_constant(value),

                    Op::ReadVariable             => self.read_variable(),

                    Op::WriteVariable            => self.write_variable(),

                    Op::Execute(value)           => self.execute_value(value),

                    Op::PushConstantValue(value) => self.push_constant_value(value),

                    Op::MarkLoopExit(value) =>
                        {
                            let computed = self.absolute_index(pc, value);

                            match computed
                            {
                                Ok(absolute_index) =>
                                    {
                                        loop_locations.push(( pc + 1, absolute_index));
                                        Ok(())
                                    },
                                Err(error) => Err(error)
                            }
                        },

                    Op::UnmarkLoopExit =>
                        {
                            if loop_locations.is_empty()
                            {
                                script_error_str(self, "Unbalanced loop exit marker.")
                            }
                            else
                            {
                                let _ = loop_locations.pop();
                                Ok(())
                            }
                        },

                    Op::MarkCatch(value) =>
                        {
                            let computed = self.absolute_index(pc, value);

                            match computed
                            {
                                Ok(absolute_index) =>
                                    {
                                        catch_locations.push(absolute_index);
                                        Ok(())
                                    },
                                Err(error) => Err(error)
                            }
                        },

                    Op::UnmarkCatch =>
                        {
                            if catch_locations.is_empty()
                            {
                                script_error_str(self, "Unbalanced catch exit marker.")
                            }
                            else
                            {
                                let _ = catch_locations.pop();
                                Ok(())
                            }
                        },

                    Op::MarkContext =>
                        {
                            self.mark_context();
                            contexts += 1;

                            Ok(())
                        },

                    Op::ReleaseContext =>
                        {
                            if contexts == 0
                            {
                                script_error_str(self, "Unbalanced context release detected.")
                            }
                            else
                            {
                                contexts -= 1;
                                Ok(())
                            }
                        },

                    Op::Jump(value) =>
                        {
                            let computed = self.absolute_index(pc, value);

                            match computed
                            {
                                Ok(absolute_index) =>
                                    {
                                        pc = absolute_index - 1;
                                        Ok(())
                                    },
                                Err(error) => Err(error)
                            }
                        },

                    Op::JumpIfZero(value)    => self.jump_if_match(&mut pc, value, false),

                    Op::JumpIfNotZero(value) => self.jump_if_match(&mut pc, value, true),

                    Op::JumpLoopStart =>
                        {
                            if loop_locations.is_empty()
                            {
                                script_error_str(self, "JumpLoopStart outside of loop.")
                            }
                            else
                            {
                                // Jump to the start of the marked loop.
                                let ( start, _ ) = loop_locations[loop_locations.len() - 1];

                                // Account for the increment that still happens at the end of the
                                //  loop.
                                pc = start - 1;
                                Ok(())
                            }
                        },

                    Op::JumpLoopExit =>
                        {
                            if loop_locations.is_empty()
                            {
                                script_error_str(self, "JumpLoopExit outside of loop.")
                            }
                            else
                            {
                                // Jump to the end of the marked loop.
                                let ( _, end ) = loop_locations[loop_locations.len() - 1];

                                // Account for the increment that still happens at the end of the
                                //  loop.
                                pc = end - 1;
                                Ok(())
                            }
                        },

                    Op::JumpTarget(_) =>
                        {
                            // Nothing to do here.  This instruction just acts as a landing pad for
                            // the jump instructions.
                            Ok(())
                        }
            };

            // If the instruction was not successful we need to clean up and report the error.
            if let Err(script_error) = result.clone()
            {
                if let Some(catch_index) = catch_locations.pop()
                {
                    pc = catch_index - 1;
                    self.push(script_error.to_string().to_value());
                }
                else
                {
                    if call_stack_pushed
                    {
                        self.call_stack_pop()?;
                    }

                    cleanup_contexts(self, contexts, false)?;
                    return result;
                }
            }
            else if call_stack_pushed
            {
                self.call_stack_pop()?;
                call_stack_pushed = false;
            }

            // Move on to the next instruction.
            pc = pc + 1;
        }

        // Make sure that the contexts are balanced.  Return an error if they are not.
        cleanup_contexts(self, contexts, true)?;

        Ok(())
    }
}


impl WordManagement for SorthInterpreter
{
    fn current_location(&self) -> &Option<SourceLocation>
    {
        &self.current_location
    }

    fn add_word(&mut self,
                file: String,
                line: usize,
                column: usize,
                name: String,
                handler: Rc<WordHandler>,
                description: String,
                signature: String,
                runtime: WordRuntime,
                visibility: WordVisibility,
                word_type: WordType)
    {
        let mut word_info = WordInfo::new();
        let location = SourceLocation::new_from_info(&file, line, column);

        let info = WordHandlerInfo::new(name.clone(), location, handler);
        let index = self.word_handlers.insert(info);

        word_info.name = name.clone();
        word_info.description = description;
        word_info.signature = signature;
        word_info.runtime = runtime;
        word_info.visibility = visibility;
        word_info.word_type = word_type;
        word_info.handler_index = index;

        self.dictionary.insert(name, word_info);
    }

    fn find_word(&self, word: &String) -> Option<&WordInfo>
    {
        self.dictionary.try_get(word)
    }

    fn word_handler_info(&self, index: usize) -> Option<&WordHandlerInfo>
    {
        if index >= self.word_handlers.len()
        {
            return None;
        }

        Some(&self.word_handlers[index])
    }

    fn inverse_name_list(&self) -> Vec<String>
    {
        Vec::new()
    }

    fn execute_word_handler(&mut self,
                            location: &Option<SourceLocation>,
                            word_handler_info: &WordHandlerInfo) -> error::Result<()>
    {
        self.current_location = location.clone();

        let location = if let Some(location) = location
            {
                location.clone()
            }
            else
            {
                SourceLocation::new()
            };

        self.call_stack.push(CallItem::new(word_handler_info.name.clone(), location));

        let result = (*word_handler_info.handler)(self);

        let _ = self.call_stack.pop();

        result
    }

    fn execute_word(&mut self,
                    location: &Option<SourceLocation>,
                    word: &WordInfo) -> error::Result<()>
    {

        let handler_info = self.word_handler_info(word.handler_index);

        if let Some(handler_info) = handler_info
        {
            self.execute_word_handler(location, &handler_info.clone())
        }
        else
        {
            script_error(self, format!("Handler for word {}, ({}) not found.",
                                       word.name,
                                       word.handler_index))
        }
    }

    fn execute_word_named(&mut self,
                          location: &Option<SourceLocation>,
                          word: &String) -> error::Result<()>
    {
        let word_info = self.dictionary.try_get(word);

        if let Some(word_info) = word_info
        {
            self.execute_word(location, &word_info.clone())
        }
        else
        {
            script_error(self, format!("Word {} not found.", word))
        }
    }

    fn execute_word_index(&mut self,
                          location: &Option<SourceLocation>,
                          index: usize) -> error::Result<()>
    {
        let handler_info = self.word_handler_info(index);

        if let Some(handler_info) = handler_info
        {
            self.execute_word_handler(location, &handler_info.clone())
        }
        else
        {
            script_error(self, format!("Word handler index {} not found.", index))
        }
    }

    fn call_stack(&self) -> &CallStack
    {
        &self.call_stack
    }

    fn call_stack_push(&mut self, name: String, location: SourceLocation)
    {
        self.call_stack.push(CallItem::new(name.clone(), location));
    }

    fn call_stack_pop(&mut self) -> error::Result<()>
    {
        if self.call_stack.is_empty()
        {
            script_error_str(self, "Call stack underflow.")?;
        }

        self.call_stack.pop();
        Ok(())
    }
}


impl ThreadManagement for SorthInterpreter
{
}


impl SorthInterpreter
{
    pub fn new() -> SorthInterpreter
    {
        SorthInterpreter
            {
                max_depth: 0,

                search_paths: Vec::new(),

                stack: Vec::new(),

                current_location: None,
                call_stack: CallStack::new(),

                data_definitions: DataDefinitionList::new(),

                dictionary: Dictionary::new(),
                word_handlers: WordList::new(),

                variables: VariableList::new(),

                constructors: CodeConstructorList::new()
            }
    }
}
