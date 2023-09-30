use crate::msg::{
    ERROR_CANNOT_COMMIT_BATCH,
};
use crate::shared::utils::principal_not_equal;
use candid::Principal;
use ic_cdk::api::time;
use std::collections::HashMap;

use crate::storage::constants::{
    ASSET_ENCODING_NO_COMPRESSION,  ENCODING_CERTIFICATION_ORDER,
};
use crate::storage::runtime::{
    clear_batch as clear_runtime_batch, clear_expired_batches as clear_expired_runtime_batches,
    clear_expired_chunks as clear_expired_runtime_chunks,
 get_batch as get_runtime_batch,
    get_chunk as get_runtime_chunk,
    insert_batch as insert_runtime_batch, insert_chunk as insert_runtime_chunk,
};
use crate::storage::state::{
    get_asset as get_state_asset,
    get_public_asset as get_state_public_asset,
    insert_asset as insert_state_asset,
};
use crate::storage::types::http_request::{MapUrl, PublicAsset};
use crate::storage::types::interface::{CommitBatch, InitAssetKey, UploadChunk};
use crate::storage::types::state::FullPath;
use crate::storage::types::store::{Asset, AssetEncoding, AssetKey, Batch, Chunk};
use crate::storage::url::{map_alternative_paths, map_url};

///
/// Getter, list and delete
///

pub fn get_public_asset_for_url(url: String) -> Result<PublicAsset, &'static str> {
    if url.is_empty() {
        return Err("No url provided.");
    }

    // The certification considers, and should only, the path of the URL. If query parameters, these should be omitted in the certificate.
    // Likewise the memory contains only assets indexed with their respective path.
    // e.g.
    // url: /hello/something?param=123
    // path: /hello/something

    let MapUrl { path, token } = map_url(&url)?;
    let alternative_paths = map_alternative_paths(&path);

    // ⚠️ Limitation: requesting an url without extension try to resolve first a corresponding asset
    // e.g. /.well-known/hello -> try to find /.well-known/hello.html
    // Therefore if a file without extension is uploaded to the storage, it is important to not upload an .html file with the same name next to it or a folder/index.html

    for alternative_path in alternative_paths {
        let asset: Option<Asset> = get_public_asset(alternative_path, token.clone());

        // We return the first match
        match asset {
            None => (),
            Some(_) => {
                return Ok(PublicAsset { url: path, asset });
            }
        }
    }

    // We return the asset that matches the effective path
    let asset: Option<Asset> = get_public_asset(path.clone(), token.clone());

    match asset {
        None => (),
        Some(_) => {
            return Ok(PublicAsset { url: path, asset });
        }
    }

    Ok(PublicAsset {
        url: path,
        asset: None,
    })
}

pub fn get_public_asset(full_path: FullPath, token: Option<String>) -> Option<Asset> {
    let asset = get_state_public_asset(&full_path);

    match asset {
        None => None,
        Some(asset) => match &asset.key.token {
            None => Some(asset.clone()),
            Some(asset_token) => get_token_protected_asset(&asset, asset_token, token),
        },
    }
}

fn get_token_protected_asset(
    asset: &Asset,
    asset_token: &String,
    token: Option<String>,
) -> Option<Asset> {
    match token {
        None => None,
        Some(token) => {
            if &token == asset_token {
                return Some(asset.clone());
            }

            None
        }
    }
}

///
/// Upload batch and chunks
///

const BATCH_EXPIRY_NANOS: u64 = 300_000_000_000;

static mut NEXT_BATCH_ID: u128 = 0;
static mut NEXT_CHUNK_ID: u128 = 0;

pub fn create_batch(caller: Principal, init: InitAssetKey) -> Result<u128, String> {
    secure_create_batch_impl(caller, init)
}

pub fn create_chunk(caller: Principal, chunk: UploadChunk) -> Result<u128, &'static str> {
    create_chunk_impl(caller, chunk)
}

pub fn commit_batch(caller: Principal, commit_batch: CommitBatch) -> Result<(), String> {
    commit_batch_impl(caller, commit_batch)
}

fn secure_create_batch_impl(caller: Principal, init: InitAssetKey) -> Result<u128, String> {
    // Assert supported encoding type
    get_encoding_type(&init.encoding_type)?;

    Ok(create_batch_impl(caller, init))
}

fn create_batch_impl(
    caller: Principal,
    InitAssetKey {
        token,
        name,
        collection,
        encoding_type,
        full_path,
        description,
    }: InitAssetKey,
) -> u128 {
    let now = time();

    unsafe {
        clear_expired_batches();

        NEXT_BATCH_ID += 1;

        let key: AssetKey = AssetKey {
            full_path,
            collection,
            owner: caller,
            token,
            name,
            description,
        };

        insert_runtime_batch(
            &NEXT_BATCH_ID,
            Batch {
                key,
                expires_at: now + BATCH_EXPIRY_NANOS,
                encoding_type,
            },
        );

        NEXT_BATCH_ID
    }
}

fn create_chunk_impl(
    caller: Principal,
    UploadChunk {
        batch_id,
        content,
        order_id,
    }: UploadChunk,
) -> Result<u128, &'static str> {
    let batch = get_runtime_batch(&batch_id);

    match batch {
        None => Err("Batch not found."),
        Some(b) => {
            if principal_not_equal(caller, b.key.owner) {
                return Err("Bach initializer does not match chunk uploader.");
            }

            let now = time();

            // Update batch to extend expires_at
            insert_runtime_batch(
                &batch_id,
                Batch {
                    key: b.key.clone(),
                    expires_at: now + BATCH_EXPIRY_NANOS,
                    encoding_type: b.encoding_type,
                },
            );

            unsafe {
                NEXT_CHUNK_ID += 1;

                insert_runtime_chunk(
                    &NEXT_CHUNK_ID,
                    Chunk {
                        batch_id,
                        content,
                        order_id: order_id.unwrap_or(NEXT_CHUNK_ID),
                    },
                );

                Ok(NEXT_CHUNK_ID)
            }
        }
    }
}

fn commit_batch_impl(caller: Principal, commit_batch: CommitBatch) -> Result<(), String> {
    let batch = get_runtime_batch(&commit_batch.batch_id);

    match batch {
        None => Err(ERROR_CANNOT_COMMIT_BATCH.to_string()),
        Some(b) => {
            let asset = secure_commit_chunks(caller, commit_batch, &b);
            Ok(())
        }
    }
}

fn secure_commit_chunks(
    caller: Principal,

    commit_batch: CommitBatch,
    batch: &Batch,
) -> Result<Asset, String> {
    // The one that started the batch should be the one that commits it
    if principal_not_equal(caller, batch.key.owner) {
        return Err(ERROR_CANNOT_COMMIT_BATCH.to_string());
    }

    let current = get_state_asset(&batch.key.full_path);

    match current {
        None => commit_chunks(commit_batch, batch),
        Some(current) => secure_commit_chunks_update(caller, commit_batch, batch, current),
    }
}

fn secure_commit_chunks_update(
    caller: Principal,

    commit_batch: CommitBatch,
    batch: &Batch,

    current: Asset,
) -> Result<Asset, String> {
    // The collection of the existing asset should be the same as the one we commit
    if batch.key.collection != current.key.collection {
        return Err("Provided collection does not match existing collection.".to_string());
    }

    commit_chunks(commit_batch, batch)
}

fn commit_chunks(
    CommitBatch {
        chunk_ids,
        batch_id,
        headers,
    }: CommitBatch,
    batch: &Batch,
) -> Result<Asset, String> {
    let now = time();

    if now > batch.expires_at {
        clear_expired_batches();
        return Err("Batch did not complete in time. Chunks cannot be committed.".to_string());
    }

    // Collect all chunks
    let mut chunks: Vec<Chunk> = vec![];

    for chunk_id in chunk_ids.iter() {
        let chunk = get_runtime_chunk(chunk_id);

        match chunk {
            None => {
                return Err("Chunk does not exist.".to_string());
            }
            Some(c) => {
                if batch_id != c.batch_id {
                    return Err("Chunk not included in the provided batch.".to_string());
                }

                chunks.push(c);
            }
        }
    }

    // Sort with ordering
    chunks.sort_by(|a, b| a.order_id.cmp(&b.order_id));

    let mut content_chunks: Vec<Vec<u8>> = vec![];

    // Collect content
    for c in chunks.iter() {
        content_chunks.push(c.content.clone());
    }

    if content_chunks.is_empty() {
        return Err("No chunk to commit.".to_string());
    }

    let key = batch.clone().key;

    let now = time();

    let mut asset: Asset = Asset {
        key,
        headers,
        encodings: HashMap::new(),
        created_at: now,
        updated_at: now,
    };

    if let Some(existing_asset) = get_state_asset(&batch.clone().key.full_path) {
        asset.encodings = existing_asset.encodings.clone();
        asset.created_at = existing_asset.created_at;
    }

    let encoding_type = get_encoding_type(&batch.encoding_type)?;

    let encoding = AssetEncoding::from(&content_chunks);

    asset.encodings.insert(encoding_type, encoding);

    insert_state_asset(&batch.clone().key.full_path, &asset);

    clear_runtime_batch(&batch_id, &chunk_ids);

    Ok(asset)
}

fn get_encoding_type(encoding_type: &Option<String>) -> Result<String, &'static str> {
    let provided_type = encoding_type
        .clone()
        .unwrap_or_else(|| ASSET_ENCODING_NO_COMPRESSION.to_string());
    let matching_type = Vec::from(ENCODING_CERTIFICATION_ORDER)
        .iter()
        .any(|&e| *e == provided_type);

    if !matching_type {
        return Err("Asset encoding not supported for certification purpose.");
    }

    Ok(provided_type)
}

fn clear_expired_batches() {
    // Remove expired batches
    clear_expired_runtime_batches();

    // Remove chunk without existing batches (those we just deleted above)
    clear_expired_runtime_chunks();
}
