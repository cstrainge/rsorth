
use crate::{ lang::source_buffer::SourceLocation,
             runtime::data_structures::value::Value };



#[derive(Clone)]
pub enum Op
{
    DefVariable(Value),
    DefConstant(Value),
    ReadVariable,
    WriteVariable,
    Execute(Value),
    WordIndex(Value),
    WordExists(Value),
    PushConstantValue(Value),
    MarkLoopExit(Value),
    UnmarkLoopExit,
    MarkCatch(Value),
    UnmarkCatch,
    MarkContext,
    ReleaseContext,
    Jump(Value),
    JumpIfZero(Value),
    JumpIfNotZero(Value),
    JumpLoopStart,
    JumpLoopExit,
    JumpTarget(Value)
}



#[derive(Clone)]
pub struct Instruction
{
    pub location: Option<SourceLocation>,
    pub op: Op
}



pub type ByteCode = Vec<Instruction>;



pub struct CodeConstructor
{
    //
}
