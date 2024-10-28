
/// This trait is used to mark and release contexts used by the interpreter.  When a context is
/// released all of the data stored in that contexts are also released.
///
/// Contexts act as a stack.  When a new context is marked it is pushed onto the stack.  When it's
/// released it is popped off and freed.
///
/// In the mean time all data added to all contexts are available to the interpreter and user code.
///
/// That is all available contexts act as a single contiguous view of the data stricture's data.
pub trait ContextualData
{
    /// Mark a new context.  Any data added to the context after this point will be released when
    /// the corresponding release_context is called.
    fn mark_context(&mut self);

    /// Release the current context.  All data added to the context since the last mark will be
    /// released.
    fn release_context(&mut self);
}
