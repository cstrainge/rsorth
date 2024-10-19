
pub trait ContextualData
{
    fn mark_context(&mut self);
    fn release_context(&mut self);
}
