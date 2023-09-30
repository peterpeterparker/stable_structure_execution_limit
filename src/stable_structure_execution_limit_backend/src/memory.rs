use crate::types::memory::Memory;
use crate::types::state::{StableState, State};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::StableBTreeMap;
use std::cell::RefCell;

const ASSETS: MemoryId = MemoryId::new(2);

thread_local! {
    pub static STATE: RefCell<State> = RefCell::default();

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

fn get_memory_assets() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(ASSETS))
}

pub fn init_stable_state() -> StableState {
    StableState {
        assets: StableBTreeMap::init(get_memory_assets()),
    }
}
