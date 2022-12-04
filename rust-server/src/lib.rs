pub mod buffer;
pub mod error;
pub mod message;
pub mod stdio;
pub mod vfs;

use crossbeam_channel::{Receiver, Sender};
use error::ProtocolError;
use lsp_types::{OneOf, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};
use message::{ErrorCode, Message, RequestId, Response};

pub fn initialize(
    sender: &Sender<Message>,
    receiver: &Receiver<Message>,
) -> Result<serde_json::Value, ProtocolError> {
    let (id, params) = (|| {
        loop {
            match receiver.recv() {
                Ok(Message::Request(req)) if req.is_initialize() => {
                    return Ok((req.id, req.params))
                }
                // Respond to non-initialize requests with ServerNotInitialized
                Ok(Message::Request(req)) => {
                    let resp = Response::new_err(
                        req.id.clone(),
                        ErrorCode::ServerNotInitialized as i32,
                        format!("expected initialize request, got {:?}", req),
                    );
                    sender.send(resp.into()).unwrap();
                }
                Ok(msg) => {
                    return Err(ProtocolError(format!(
                        "expected initialize request, got {:?}",
                        msg
                    )))
                }
                Err(e) => {
                    return Err(ProtocolError(format!(
                        "expected initialize request, got error: {}",
                        e
                    )))
                }
            };
        }
    })()?;

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    };

    let initialize_data = serde_json::json!({
        "capabilities": server_capabilities,
    });

    initialize_finish(&sender, &receiver, id, initialize_data)?;

    Ok(params)
}

pub fn initialize_finish(
    sender: &Sender<Message>,
    receiver: &Receiver<Message>,
    initialize_id: RequestId,
    initialize_result: serde_json::Value,
) -> Result<(), ProtocolError> {
    let resp = Response::new_ok(initialize_id, initialize_result);
    sender.send(resp.into()).unwrap();
    match &receiver.recv() {
        Ok(Message::Notification(n)) if n.is_initialized() => (),
        Ok(msg) => {
            return Err(ProtocolError(format!(
                "expected Message::Notification, got: {:?}",
                msg,
            )))
        }
        Err(e) => {
            return Err(ProtocolError(format!(
                "expected initialized notification, got error: {}",
                e,
            )))
        }
    }
    Ok(())
}
