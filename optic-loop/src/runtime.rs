use crate::Game;

pub trait Runtime {
    fn start(&mut self, game: &mut Game);
    fn update(&mut self, game: &mut Game);
    fn end(&mut self, _game: &mut Game) {}
}
