use crate::memory::init_stable_state;
use crate::types::state::{RuntimeState, State};

impl Default for State {
    fn default() -> Self {
        Self {
            stable: init_stable_state(),
            runtime: RuntimeState::default(),
        }
    }
}
