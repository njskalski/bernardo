use crate::io::loading_state::LoadingState;

pub trait CodeResultsProvider {
    fn loading_state(&self) -> LoadingState;

    // fn items(&self) -> Box<dyn Iterator<Item=>>
}