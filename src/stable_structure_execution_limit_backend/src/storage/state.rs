use crate::memory::STATE;
use crate::storage::types::state::{AssetsStable, FullPath, StableFullPath};
use crate::storage::types::store::Asset;

/// Assets

pub fn get_public_asset(full_path: &FullPath) -> Option<Asset> {
    STATE.with(|state| get_asset_stable(full_path, &state.borrow().stable.assets))
}

pub fn get_asset(full_path: &FullPath) -> Option<Asset> {
    STATE.with(|state| get_asset_stable(full_path, &state.borrow().stable.assets))
}

pub fn insert_asset(full_path: &FullPath, asset: &Asset) {
    STATE.with(|state| insert_asset_stable(full_path, asset, &mut state.borrow_mut().stable.assets))
}

// Get

fn get_asset_stable(full_path: &FullPath, assets: &AssetsStable) -> Option<Asset> {
    assets.get(&stable_full_path(full_path))
}

// Insert

fn insert_asset_stable(full_path: &FullPath, asset: &Asset, assets: &mut AssetsStable) {
    assets.insert(stable_full_path(full_path), asset.clone());
}

fn stable_full_path(full_path: &FullPath) -> StableFullPath {
    StableFullPath {
        full_path: full_path.clone(),
    }
}
