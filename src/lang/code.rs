
use std::{ cmp::Ordering,
           fmt::{ self, Display, Formatter },
           hash::{ Hash, Hasher } };
use crate::{ lang::source_buffer::SourceLocation,
             runtime::{ interpreter::Interpreter,
                        data_structures::value::Value } };



#[derive(Clone)]
pub enum Op
{
    DefVariable(Value),
    DefConstant(Value),
    ReadVariable,
    WriteVariable,
    Execute(Value),
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



#[derive(Clone, PartialEq, PartialOrd)]
pub struct Instruction
{
    pub location: Option<SourceLocation>,
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



impl Display for Instruction
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        fn flt(value: &Value) -> String
        {
            match value
            {
                Value::String(text) => Value::stringify(&text),
                _ => format!("{}", value)
            }
        }

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



pub type ByteCode = Vec<Instruction>;



impl Instruction
{
    pub fn new(location: Option<SourceLocation>, op: Op) -> Instruction
    {
        Instruction
            {
                location,
                op
            }
    }
}



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
