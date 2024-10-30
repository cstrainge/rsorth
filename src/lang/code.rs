
use std::{ cmp::Ordering,
           collections::VecDeque,
           fmt::{ self,
                  Display,
                  Formatter },
           hash::{ Hash,
                   Hasher } };
use crate::{ lang::source_buffer::SourceLocation,
             runtime::{ interpreter::Interpreter,
                        data_structures::value::Value } };



/// The operations that can be performed by the Strange Forth virtual machine.
#[derive(Clone, Eq)]
pub enum Op
{
    /// Define a variable in the current context.  The value is expected to be the new variable's
    /// name.
    DefVariable(Value),

    /// Define a constant in the current context.  The value is expected to be the new constant's
    /// name.  The value is pulled from the stack.
    DefConstant(Value),

    /// Read a value from a variable in the current or previous contexts.  This instruction expects
    /// the top of the stack to contain the variable's index.
    ReadVariable,

    /// Write to a variable in the current or previous contexts.  This instruction expects the top
    /// of the stack to contain the variable's index and the second item on the stack to contain the
    /// value to write.
    WriteVariable,

    /// Execute a word in the current or previous contexts.  This instruction expects the value to
    /// be either the word's name or the word's index.
    Execute(Value),

    /// Push a constant value onto the stack.  This instruction expects the value to be the constant
    /// value to push.  A deep clone is performed to make sure user code can not modify the
    /// original.
    PushConstantValue(Value),

    /// Mark the next instruction as the top of the loop and the value is expected to be a relative
    /// index to the loop's exit.
    ///
    /// During compilation the value is the target label's name.  The location of the loop start is
    /// computed at runtime as the instruction following this one.  At the end of the compile phase
    /// the value is resolved to be the relative index to the target instruction.
    MarkLoopExit(Value),

    /// Unmark the last loop start/exit pair.
    UnmarkLoopExit,

    /// Mark the location of a catch block.  The value is expected to the the relative index to the
    /// catch block's first instruction.
    ///
    /// During compilation the value is the catch block's label name.  At the end of the compile
    /// phase the value is resolved to be the relative index to the target instruction.
    MarkCatch(Value),

    /// Unmark the last catch block.  If an exception is thrown a prior, (if it exists,) will be
    /// the one to catch it.  Otherwise the execution stack will be unwound to the any previous
    /// word's catch block.
    UnmarkCatch,

    /// Mark a new interpreter context.  Any words or variables created will be in this new context
    /// until it is released.  It is expected that the context will be released before the current
    /// word exits.  It is a runtime error to have unbalanced context acquire/release pairs.
    MarkContext,

    /// Release the current interpreter context.  Any words or variables created within this context
    /// will be lost.  It is a runtime error to have unbalanced context acquire/release pairs.
    ReleaseContext,

    /// Jump to a new instruction.  The value is expected to be the relative index to the target
    /// instruction.
    ///
    /// During compilation the value is the target label's name.  At the end of the compile phase
    /// the value is resolved to be the relative index to the target instruction.
    Jump(Value),

    /// Jump to a new instruction if the top of the stack is false.  The encoded value is expected
    /// to be the relative index to the target instruction.
    ///
    /// During compilation the value is the target label's name.  At the end of the compile phase
    /// the value is resolved to be the relative index to the target instruction.
    JumpIfZero(Value),

    /// Jump to a new instruction if the top of the stack is a true value.  The encoded value is
    /// expected to be the relative index to the target instruction.
    ///
    /// During compilation the value is the target label's name.  At the end of the compile phase
    /// the value is resolved to be the relative index to the target instruction.
    JumpIfNotZero(Value),

    /// Jump to the start of a marked loop.  This instruction is used to implement the loop continue
    /// statement.  It is expected that the loop start was marked with a MarkLoopExit instruction
    /// prior to this instruction.  A runtime error will occur if the loop was not marked.  Loops
    /// can be nested and the last loop marked will be the one that is jumped to.
    JumpLoopStart,

    /// Jump to the end of a marked loop and exit it.  This instruction is used to implement the
    /// loop exit statement.  It is expected that the loop end was marked with a MarkLoopExit
    /// instruction prior to this instruction.  A runtime error will occur if the loop was not
    /// marked.  Loops can be nested and the last loop marked will be the one that is exited.
    JumpLoopExit,

    /// The target of a jump instruction.  This instruction is a no-op and is used to mark a target
    /// for one of the jump instructions.  During compilation the value is the target's name.  At
    /// runtime the value is set to None and ignored.
    JumpTarget(Value)
}



impl PartialEq for Op
{
    fn eq(&self, other: &Self) -> bool
    {
        match ( self, other )
        {
            ( Op::DefVariable(a),       Op::DefVariable(b)       ) => a == b,
            ( Op::DefConstant(a),       Op::DefConstant(b)       ) => a == b,
            ( Op::ReadVariable,         Op::ReadVariable         ) => true,
            ( Op::WriteVariable,        Op::WriteVariable        ) => true,
            ( Op::Execute(a),           Op::Execute(b)           ) => a == b,
            ( Op::PushConstantValue(a), Op::PushConstantValue(b) ) => a == b,
            ( Op::MarkLoopExit(a),      Op::MarkLoopExit(b)      ) => a == b,
            ( Op::UnmarkLoopExit,       Op::UnmarkLoopExit       ) => true,
            ( Op::MarkCatch(a),         Op::MarkCatch(b)         ) => a == b,
            ( Op::UnmarkCatch,          Op::UnmarkCatch          ) => true,
            ( Op::MarkContext,          Op::MarkContext          ) => true,
            ( Op::ReleaseContext,       Op::ReleaseContext       ) => true,
            ( Op::Jump(a),              Op::Jump(b)              ) => a == b,
            ( Op::JumpIfZero(a),        Op::JumpIfZero(b)        ) => a == b,
            ( Op::JumpIfNotZero(a),     Op::JumpIfNotZero(b)     ) => a == b,
            ( Op::JumpLoopStart,        Op::JumpLoopStart        ) => true,
            ( Op::JumpLoopExit,         Op::JumpLoopExit         ) => true,
            ( Op::JumpTarget(a),        Op::JumpTarget(b)        ) => a == b,

            _ => false
        }
    }
}


impl PartialOrd for Op
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        match ( self, other )
        {
            ( Op::DefVariable(a),       Op::DefVariable(b)       ) => a.partial_cmp(b),
            ( Op::DefConstant(a),       Op::DefConstant(b)       ) => a.partial_cmp(b),
            ( Op::ReadVariable,         Op::ReadVariable         ) => Some(Ordering::Equal),
            ( Op::WriteVariable,        Op::WriteVariable        ) => Some(Ordering::Equal),
            ( Op::Execute(a),           Op::Execute(b)           ) => a.partial_cmp(b),
            ( Op::PushConstantValue(a), Op::PushConstantValue(b) ) => a.partial_cmp(b),
            ( Op::MarkLoopExit(a),      Op::MarkLoopExit(b)      ) => a.partial_cmp(b),
            ( Op::UnmarkLoopExit,       Op::UnmarkLoopExit       ) => Some(Ordering::Equal),
            ( Op::MarkCatch(a),         Op::MarkCatch(b)         ) => a.partial_cmp(b),
            ( Op::UnmarkCatch,          Op::UnmarkCatch          ) => Some(Ordering::Equal),
            ( Op::MarkContext,          Op::MarkContext          ) => Some(Ordering::Equal),
            ( Op::ReleaseContext,       Op::ReleaseContext       ) => Some(Ordering::Equal),
            ( Op::Jump(a),              Op::Jump(b)              ) => a.partial_cmp(b),
            ( Op::JumpIfZero(a),        Op::JumpIfZero(b)        ) => a.partial_cmp(b),
            ( Op::JumpIfNotZero(a),     Op::JumpIfNotZero(b)     ) => a.partial_cmp(b),
            ( Op::JumpLoopStart,        Op::JumpLoopStart        ) => Some(Ordering::Equal),
            ( Op::JumpLoopExit,         Op::JumpLoopExit         ) => Some(Ordering::Equal),
            ( Op::JumpTarget(a),        Op::JumpTarget(b)        ) => a.partial_cmp(b),

            _ => None
        }
    }
}


impl Hash for Op
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        match self
        {
            Op::DefVariable(value)       => {  0.hash(state); value.hash(state); },
            Op::DefConstant(value)       => {  1.hash(state); value.hash(state); },
            Op::ReadVariable             =>    2.hash(state),
            Op::WriteVariable            =>    3.hash(state),
            Op::Execute(value)           => {  4.hash(state); value.hash(state); },
            Op::PushConstantValue(value) => {  7.hash(state); value.hash(state); },
            Op::MarkLoopExit(value)      => {  8.hash(state); value.hash(state); },
            Op::UnmarkLoopExit           =>    9.hash(state),
            Op::MarkCatch(value)         => { 10.hash(state); value.hash(state); },
            Op::UnmarkCatch              =>   11.hash(state),
            Op::MarkContext              =>   12.hash(state),
            Op::ReleaseContext           =>   13.hash(state),
            Op::Jump(value)              => { 14.hash(state); value.hash(state); },
            Op::JumpIfZero(value)        => { 15.hash(state); value.hash(state); },
            Op::JumpIfNotZero(value)     => { 16.hash(state); value.hash(state); },
            Op::JumpLoopStart            =>   17.hash(state),
            Op::JumpLoopExit             =>   18.hash(state),
            Op::JumpTarget(value)        => { 19.hash(state); value.hash(state); }
        }
    }
}



/// Represents a single instruction in the Strange Forth virtual machine.
#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct Instruction
{
    /// Location in the source code this instruction was generated from.  Instructions generated by
    /// user code will not have a location.
    pub location: Option<SourceLocation>,

    /// The operation to perform and optionally it's value as defined by the Op enum.
    pub op: Op
}



impl Hash for Instruction
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        self.location.hash(state);
        self.op.hash(state);
    }
}


/// Allow for pretty printing of the instruction and it's value.
impl Display for Instruction
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        // Format the value as a string, doing things like changing newlines to '\n' and escaping
        // other special characters.  The string is then enclosed in quotes.
        fn flt(value: &Value) -> String
        {
            match value
            {
                Value::String(text) => Value::stringify(&text),
                _ => format!("{}", value)
            }
        }

        // Filter a JumpTarget's value.  If it is still the label name then it is printed.  If it
        // has been resolved then the None value is not printed.
        fn jt(value: &Value) -> String
        {
            match value
            {
                Value::None => "".to_string(),
                _ => format!("{}", value)
            }
        }

        match &self.op
        {
            Op::DefVariable(value)       => write!(f, "DefVariable       {}", value),
            Op::DefConstant(value)       => write!(f, "DefConstant       {}", value),
            Op::ReadVariable             => write!(f, "ReadVariable"),
            Op::WriteVariable            => write!(f, "WriteVariable"),
            Op::Execute(value)           => write!(f, "Execute           {}", value),
            Op::PushConstantValue(value) => write!(f, "PushConstantValue {}", flt(&value)),
            Op::MarkLoopExit(value)      => write!(f, "MarkLoopExit      {}", value),
            Op::UnmarkLoopExit           => write!(f, "UnmarkLoopExit"),
            Op::MarkCatch(value)         => write!(f, "MarkCatch         {}", value),
            Op::UnmarkCatch              => write!(f, "UnmarkCatch"),
            Op::MarkContext              => write!(f, "MarkContext"),
            Op::ReleaseContext           => write!(f, "ReleaseContext"),
            Op::Jump(value)              => write!(f, "Jump              {}", value),
            Op::JumpIfZero(value)        => write!(f, "JumpIfZero        {}", value),
            Op::JumpIfNotZero(value)     => write!(f, "JumpIfNotZero     {}", value),
            Op::JumpLoopStart            => write!(f, "JumpLoopStart"),
            Op::JumpLoopExit             => write!(f, "JumpLoopExit"),
            Op::JumpTarget(value)        => write!(f, "JumpTarget        {}", jt(&value)),
        }
    }
}



/// A collection of instructions that make up a Strange Forth word, or script toplevel code.  We use
/// a VecDeque to allow for efficient addition of instructions at the beginning and end of the
/// collection.
pub type ByteCode = VecDeque<Instruction>;



impl Instruction
{
    /// Create a new instruction with a location and operation.
    pub fn new(location: Option<SourceLocation>, op: Op) -> Instruction
    {
        Instruction
            {
                location,
                op
            }
    }
}



/// Pretty print the byte code for debugging purposes.
pub fn pretty_print_code(_interpreter: Option<&dyn Interpreter>, code: &ByteCode) -> String
{
    use std::fmt::Write;

    let mut result = String::with_capacity(code.len() * 20);

    for ( index, instruction ) in code.iter().enumerate()
    {
        writeln!(&mut result, "{:4}: {}", index, instruction)
            .expect("Writing to String should never fail.");
    }

    result
}
