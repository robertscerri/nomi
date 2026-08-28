pub trait FocusTarget {
    fn focus(&mut self);
    fn blur(&mut self);
    fn is_focused(&self) -> bool;
}
