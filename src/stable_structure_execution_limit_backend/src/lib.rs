mod impls;
mod memory;
mod msg;
mod shared;
mod storage;
mod types;

use crate::memory::{init_stable_state, STATE};
use crate::storage::http::{
    build_encodings, build_headers, create_token, error_response, streaming_strategy,
};
use crate::storage::store::{
    commit_batch, create_batch, create_chunk, get_public_asset, get_public_asset_for_url,
};
use crate::storage::types::http::{
    HttpRequest, HttpResponse, StreamingCallbackHttpResponse, StreamingCallbackToken,
};
use crate::storage::types::http_request::PublicAsset;
use crate::storage::types::interface::{
    CommitBatch, InitAssetKey, InitUploadResult, UploadChunk, UploadChunkResult,
};
use crate::storage::types::store::Asset;
use crate::types::state::{RuntimeState, State};
use ic_cdk::api::{caller, trap};
use ic_cdk_macros::{export_candid, init, query, update};

#[init]
fn init() {
    STATE.with(|state| {
        *state.borrow_mut() = State {
            stable: init_stable_state(),
            runtime: RuntimeState::default(),
        };
    });
}

///
/// Http
///

#[query]
fn http_request(
    HttpRequest {
        method,
        url,
        headers: req_headers,
        body: _,
    }: HttpRequest,
) -> HttpResponse {
    if method != "GET" {
        return error_response(405, "Method Not Allowed.".to_string());
    }

    let result = get_public_asset_for_url(url);

    match result {
        Ok(PublicAsset {
            asset,
            url: requested_url,
        }) => match asset {
            Some(asset) => {
                let encodings = build_encodings(req_headers);

                for encoding_type in encodings.iter() {
                    if let Some(encoding) = asset.encodings.get(encoding_type) {
                        let headers =
                            build_headers(&requested_url, &asset, encoding, encoding_type);

                        let Asset {
                            key,
                            headers: _,
                            encodings: _,
                            created_at: _,
                            updated_at: _,
                        } = &asset;

                        match headers {
                            Ok(headers) => {
                                return HttpResponse {
                                    body: encoding.content_chunks[0].clone(),
                                    headers: headers.clone(),
                                    status_code: 200,
                                    streaming_strategy: streaming_strategy(
                                        key,
                                        encoding,
                                        encoding_type,
                                        &headers,
                                    ),
                                }
                            }
                            Err(err) => {
                                return error_response(
                                    405,
                                    ["Permission denied. Invalid headers. ", err].join(""),
                                );
                            }
                        }
                    }
                }

                error_response(500, "No asset encoding found.".to_string())
            }
            None => error_response(404, "No asset found.".to_string()),
        },
        Err(err) => error_response(
            405,
            ["Permission denied. Cannot perform this operation. ", err].join(""),
        ),
    }
}

#[query]
fn http_request_streaming_callback(
    StreamingCallbackToken {
        token,
        headers,
        index,
        sha256: _,
        full_path,
        encoding_type,
    }: StreamingCallbackToken,
) -> StreamingCallbackHttpResponse {
    let asset = get_public_asset(full_path, token);

    match asset {
        Some(asset) => {
            let encoding = asset.encodings.get(&encoding_type);

            match encoding {
                Some(encoding) => StreamingCallbackHttpResponse {
                    token: create_token(&asset.key, index, encoding, &encoding_type, &headers),
                    body: encoding.content_chunks[index].clone(),
                },
                None => trap("Streamed asset encoding not found."),
            }
        }
        None => trap("Streamed asset not found."),
    }
}

//
// Upload
//

#[update]
fn init_asset_upload(init: InitAssetKey) -> InitUploadResult {
    let caller = caller();
    let result = create_batch(caller, init);

    match result {
        Ok(batch_id) => InitUploadResult { batch_id },
        Err(error) => trap(&error),
    }
}

#[update]
fn upload_asset_chunk(chunk: UploadChunk) -> UploadChunkResult {
    let caller = caller();

    let result = create_chunk(caller, chunk);

    match result {
        Ok(chunk_id) => UploadChunkResult { chunk_id },
        Err(error) => trap(error),
    }
}

#[update]
fn commit_asset_upload(commit: CommitBatch) {
    let caller = caller();

    commit_batch(caller, commit).unwrap_or_else(|e| trap(&e));
}

/// Mgmt

#[query]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Generate did files

export_candid!();
