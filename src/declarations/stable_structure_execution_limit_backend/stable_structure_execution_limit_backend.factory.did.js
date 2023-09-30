// @ts-ignore
export const idlFactory = ({ IDL }) => {
  const CommitBatch = IDL.Record({
    batch_id: IDL.Nat,
    headers: IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    chunk_ids: IDL.Vec(IDL.Nat),
  });
  const HttpRequest = IDL.Record({
    url: IDL.Text,
    method: IDL.Text,
    body: IDL.Vec(IDL.Nat8),
    headers: IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const StreamingCallbackToken = IDL.Record({
    token: IDL.Opt(IDL.Text),
    sha256: IDL.Opt(IDL.Vec(IDL.Nat8)),
    headers: IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    index: IDL.Nat64,
    encoding_type: IDL.Text,
    full_path: IDL.Text,
  });
  const StreamingStrategy = IDL.Variant({
    Callback: IDL.Record({
      token: StreamingCallbackToken,
      callback: IDL.Func([], [], ["query"]),
    }),
  });
  const HttpResponse = IDL.Record({
    body: IDL.Vec(IDL.Nat8),
    headers: IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    streaming_strategy: IDL.Opt(StreamingStrategy),
    status_code: IDL.Nat16,
  });
  const StreamingCallbackHttpResponse = IDL.Record({
    token: IDL.Opt(StreamingCallbackToken),
    body: IDL.Vec(IDL.Nat8),
  });
  const InitAssetKey = IDL.Record({
    token: IDL.Opt(IDL.Text),
    collection: IDL.Text,
    name: IDL.Text,
    description: IDL.Opt(IDL.Text),
    encoding_type: IDL.Opt(IDL.Text),
    full_path: IDL.Text,
  });
  const InitUploadResult = IDL.Record({ batch_id: IDL.Nat });
  const UploadChunk = IDL.Record({
    content: IDL.Vec(IDL.Nat8),
    batch_id: IDL.Nat,
    order_id: IDL.Opt(IDL.Nat),
  });
  const UploadChunkResult = IDL.Record({ chunk_id: IDL.Nat });
  return IDL.Service({
    commit_asset_upload: IDL.Func([CommitBatch], [], []),
    http_request: IDL.Func([HttpRequest], [HttpResponse], ["query"]),
    http_request_streaming_callback: IDL.Func(
      [StreamingCallbackToken],
      [StreamingCallbackHttpResponse],
      ["query"],
    ),
    init_asset_upload: IDL.Func([InitAssetKey], [InitUploadResult], []),
    upload_asset_chunk: IDL.Func([UploadChunk], [UploadChunkResult], []),
    version: IDL.Func([], [IDL.Text], ["query"]),
  });
};
// @ts-ignore
export const init = ({ IDL }) => {
  return [];
};
