use blueos_idl::Message;

/// Decodes a command or query request, accepting an empty payload as the message default (D-12).
pub(crate) fn decode_allow_empty_body<Request: Message + Default>(
    payload: &[u8],
) -> Result<Request, String> {
    if payload.is_empty() {
        return Ok(Request::default());
    }
    Request::decode(payload).map_err(|error| error.to_string())
}

/// Decodes a command or query request; the payload must be valid CDR for `Request`.
pub(crate) fn decode_strict<Request: Message>(payload: &[u8]) -> Result<Request, String> {
    Request::decode(payload).map_err(|error| error.to_string())
}
