use crossterm::event::KeyEvent;

pub trait Interactive {
    fn handle_key(&mut self, key: KeyEvent);
}
