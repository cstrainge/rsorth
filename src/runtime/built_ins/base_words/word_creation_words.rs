
use std::rc::Rc;
use crate::{ add_native_immediate_word,
             lang::{ code::ByteCode,
                     tokenizing::Token },
             runtime::{ data_structures::dictionary::{ WordContext,
                                                       WordRuntime,
                                                       WordType,
                                                       WordVisibility },
                        error::{ self,
                                 script_error_str },
                        interpreter::Interpreter } };



/// A script defined word.
struct ScriptFunction
{
    /// The name of the word.
    name: String,

    /// The context management of the word.
    context: WordContext,

    /// The byte-code for the word.
    code: ByteCode
}


impl ScriptFunction
{
    /// Create the new ScriptFunction handler.
    pub fn new(name: String, context: WordContext, code: ByteCode) -> ScriptFunction
    {
        ScriptFunction
            {
                name,
                context,
                code
            }
    }
}


/// Implement the Fn trait for ScriptFunction to make the struct callable.
impl Fn<( &mut dyn Interpreter, )> for ScriptFunction
{
    extern "rust-call" fn call(&self, args: ( &mut dyn Interpreter, ) ) -> error::Result<()>
    {
        if let WordContext::Managed = self.context
        {
            args.0.mark_context();
        }

        let result = args.0.execute_code(&self.name, &self.code);

        if let WordContext::Managed = self.context
        {
            args.0.release_context();
        }

        result
    }
}


/// Implement the FnMut trait for ScriptFunction to make the struct callable.
impl FnMut<( &mut dyn Interpreter, )> for ScriptFunction
{
    extern "rust-call" fn call_mut(&mut self, args: ( &mut dyn Interpreter, )) -> error::Result<()>
    {
        if let WordContext::Managed = self.context
        {
            args.0.mark_context();
        }

        let result = args.0.execute_code(&self.name, &self.code);

        if let WordContext::Managed = self.context
        {
            args.0.release_context();
        }

        result
    }
}


/// Implement the FnOnce trait for the ScriptFunction to make the struct callable.
impl FnOnce<( &mut dyn Interpreter, )> for ScriptFunction
{
    type Output = error::Result<()>;

    extern "rust-call" fn call_once(self, args: ( &mut dyn Interpreter, )) -> error::Result<()>
    {
        if let WordContext::Managed = self.context
        {
            args.0.mark_context();
        }

        let result = args.0.execute_code(&self.name, &self.code);

        if let WordContext::Managed = self.context
        {
            args.0.release_context();
        }

        result
    }
}



/// Start the creation of a new word.  Pull the name of the word from the next token in the token
/// stream.
fn word_start_word(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let token = interpreter.next_token()?;
    let ( location, name ) = match token
        {
            Token::Word(location, name) => ( location, name ),
            Token::Number(location, value) => ( location, value.to_string() ),
            Token::String(_, _) =>
                {
                    return script_error_str(interpreter, "Can not use a string as a word name.");
                }
        };

    interpreter.context_mut().construction_new();

    interpreter.context_mut().construction_mut()?.name = name;
    interpreter.context_mut().construction_mut()?.location = location;

    Ok(())
}

/// End the creation of a new word and register it with the interpreter.
fn word_end_word(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let construction = interpreter.context_mut().construction_pop()?;

    let new_function = ScriptFunction::new(construction.name.clone(),
                                           construction.context,
                                           construction.code);

    interpreter.add_word(construction.location.path().clone(),
                         construction.location.line(),
                         construction.location.column(),
                         construction.name,
                         Rc::new(new_function),
                         construction.description,
                         construction.signature,
                         construction.runtime,
                         construction.visibility,
                         WordType::Scripted);

    Ok(())
}

/// Mark the current word being generated word as immediate.
fn word_immediate(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_mut().construction_mut()?.runtime = WordRuntime::Immediate;
    Ok(())
}

/// Mark the current word being generated as hidden from the directory index.
fn word_hidden(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_mut().construction_mut()?.visibility = WordVisibility::Hidden;
    Ok(())
}

/// Mark the current word being generated as not using the automatic context management.
fn word_contextless(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    interpreter.context_mut().construction_mut()?.context = WordContext::Manual;
    Ok(())
}

/// Give a description to the current word being generated.
fn word_description(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let description = interpreter.next_token_string()?;

    interpreter.context_mut().construction_mut()?.description = description;
    Ok(())
}

/// Document the signature of the current word being generated.
fn word_signature(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let signature = interpreter.next_token_string()?;

    interpreter.context_mut().construction_mut()?.signature = signature;
    Ok(())
}



/// Register the word creation words with the interpreter.
pub fn register_word_creation_words(interpreter: &mut dyn Interpreter)
{
    add_native_immediate_word!(interpreter, ":", word_start_word,
        "Start a new word definition.",
        " -- ");

    add_native_immediate_word!(interpreter, ";", word_end_word,
        "End the definition of the newly created word.",
        " -- ");

    add_native_immediate_word!(interpreter, "immediate", word_immediate,
        "Mark the new word as immediate.",
        " -- ");

    add_native_immediate_word!(interpreter, "hidden", word_hidden,
        "Mark the new word as hidden from the directory.",
        " -- ");

    add_native_immediate_word!(interpreter, "contextless", word_contextless,
        "Mark the new word as not using the automatic context management.",
        " -- ");

    add_native_immediate_word!(interpreter, "description:", word_description,
        "Give a description for the new word.",
        " -- ");

    add_native_immediate_word!(interpreter, "signature:", word_signature,
        "Document the word's signature.",
        " -- ");
}
