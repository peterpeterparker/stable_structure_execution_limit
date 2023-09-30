import type { Principal } from "@dfinity/principal";
import type { ActorMethod } from "@dfinity/agent";

export interface CommitBatch {
  batch_id: bigint;
  headers: Array<[string, string]>;
  chunk_ids: Array<bigint>;
}
export interface HttpRequest {
  url: string;
  method: string;
  body: Uint8Array | number[];
  headers: Array<[string, string]>;
}
export interface HttpResponse {
  body: Uint8Array | number[];
  headers: Array<[string, string]>;
  streaming_strategy: [] | [StreamingStrategy];
  status_code: number;
}
export interface InitAssetKey {
  token: [] | [string];
  collection: string;
  name: string;
  description: [] | [string];
  encoding_type: [] | [string];
  full_path: string;
}
export interface InitUploadResult {
  batch_id: bigint;
}
export interface StreamingCallbackHttpResponse {
  token: [] | [StreamingCallbackToken];
  body: Uint8Array | number[];
}
export interface StreamingCallbackToken {
  token: [] | [string];
  sha256: [] | [Uint8Array | number[]];
  headers: Array<[string, string]>;
  index: bigint;
  encoding_type: string;
  full_path: string;
}
export type StreamingStrategy = {
  Callback: {
    token: StreamingCallbackToken;
    callback: [Principal, string];
  };
};
export interface UploadChunk {
  content: Uint8Array | number[];
  batch_id: bigint;
  order_id: [] | [bigint];
}
export interface UploadChunkResult {
  chunk_id: bigint;
}
export interface _SERVICE {
  commit_asset_upload: ActorMethod<[CommitBatch], undefined>;
  http_request: ActorMethod<[HttpRequest], HttpResponse>;
  http_request_streaming_callback: ActorMethod<
    [StreamingCallbackToken],
    StreamingCallbackHttpResponse
  >;
  init_asset_upload: ActorMethod<[InitAssetKey], InitUploadResult>;
  upload_asset_chunk: ActorMethod<[UploadChunk], UploadChunkResult>;
  version: ActorMethod<[], string>;
}
