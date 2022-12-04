use std::error::Error;

use crossbeam_channel::{Receiver, Sender};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification, PublishDiagnostics},
    request::GotoDefinition,
    DiagnosticSeverity, GotoDefinitionResponse, InitializeParams, Position,
    PublishDiagnosticsParams, Range,
};
use rust_server::{
    error::{ExtractError, ProtocolError},
    initialize,
    message::{self, Message, RequestId, Response},
    stdio,
    vfs::{File, Vfs},
};

fn main() {
    let (sender, receiver, io_threads) = stdio::stdio_transport();
    let initialization_params = initialize(&sender, &receiver).unwrap();
    main_loop(sender, receiver, initialization_params).unwrap();
    io_threads.join().unwrap();
}

fn main_loop(
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut vfs = Vfs::new();
    eprintln!("starting example main loop");
    for msg in &receiver {
        eprintln!("got msg: {:?}", msg);
        match msg {
            Message::Request(req) => {
                if handle_shutdown(&sender, &receiver, &req)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", req);
                match cast::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        eprintln!("got gotoDefinition request #{}: {:?}", id, params);
                        let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
                // ...
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                eprintln!("got notification: {:?}", not);
                if not.method == DidOpenTextDocument::METHOD {
                    let params = cast_not::<DidOpenTextDocument>(not).unwrap();
                    vfs.add(File::new(
                        params.text_document.uri.to_string(),
                        params.text_document.language_id.clone(),
                        params.text_document.version,
                        params.text_document.text.as_str().into(),
                    ))
                } else if not.method == DidChangeTextDocument::METHOD {
                    let params = cast_not::<DidChangeTextDocument>(not).unwrap();
                    vfs.update(
                        params.text_document.uri.as_str(),
                        params.text_document.version,
                        |buffer| {
                            for change in &params.content_changes {
                                let range = change.range.unwrap();
                                buffer.update(
                                    (range.start.line as usize, range.start.character as usize),
                                    (range.end.line as usize, range.end.character as usize),
                                    &change.text,
                                );
                            }
                        },
                    );
                    // eprintln!("{:?}", vfs.get(&params.text_document.uri.as_str()).content_ref().chars().collect::<String>());
                    handle_validation(
                        &sender,
                        &receiver,
                        params.text_document.uri.as_str(),
                        &vfs.get(&params.text_document.uri.as_str())
                            .content_ref()
                            .chars()
                            .collect::<String>(),
                        params.text_document.version,
                    );
                }
            }
        }
    }
    Ok(())
}

fn cast<R>(req: message::Request) -> Result<(RequestId, R::Params), ExtractError<message::Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_not<R>(req: message::Notification) -> Result<R::Params, ExtractError<message::Notification>>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

pub fn handle_shutdown(
    sender: &Sender<Message>,
    receiver: &Receiver<Message>,
    req: &message::Request,
) -> Result<bool, ProtocolError> {
    if !req.is_shutdown() {
        return Ok(false);
    }
    let resp = Response::new_ok(req.id.clone(), ());
    let _ = sender.send(resp.into());
    match &receiver.recv_timeout(std::time::Duration::from_secs(30)) {
        Ok(Message::Notification(n)) if n.is_exit() => (),
        Ok(msg) => {
            return Err(ProtocolError(format!(
                "unexpected message during shutdown: {:?}",
                msg
            )))
        }
        Err(e) => {
            return Err(ProtocolError(format!(
                "unexpected error during shutdown: {}",
                e
            )))
        }
    }
    Ok(true)
}

pub fn handle_validation(
    sender: &Sender<Message>,
    receiver: &Receiver<Message>,
    uri: &str,
    src: &str,
    version: i32,
) {
    let diagnostics = if let Some(x) = src.find("hello") {
        vec![lsp_types::Diagnostic::new(
            Range::new(Position::new(0, 0), Position::new(0, 1)),
            Some(DiagnosticSeverity::WARNING),
            None,
            Some("ex".to_string()),
            "aaa".to_string(),
            None,
            None,
        )]
    } else {
        vec![]
    };
    let version = Some(version);
    let uri = uri.parse().unwrap();
    let params = PublishDiagnosticsParams::new(uri, diagnostics, version);
    sender
        .send(Message::Notification(message::Notification {
            method: PublishDiagnostics::METHOD.to_string(),
            params: serde_json::to_value(params).unwrap(),
        }))
        .unwrap();
}
