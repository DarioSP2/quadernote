//! QuaderNote Automation Crate (L5).
//! Provides JSON-RPC 2.0 & MCP automation server, batch JSONL execution, stdio and TCP control listeners.

use quader_doc::Notebook;
use quader_engine::{Command, CommandOutcome, EngineError, EngineSession, SearchHit};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use thiserror::Error;
use uuid::Uuid;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum AutomationError {
    #[error("Engine error: {0}")]
    Engine(#[from] EngineError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RPC error: {0}")]
    Rpc(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotebookSummary {
    pub id: Uuid,
    pub name: String,
    pub active_section_id: Uuid,
    pub active_page_id: Uuid,
    pub sections_count: usize,
    pub pages: Vec<PageSummary>,
    pub active_page_item_counts: ActivePageItemCounts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageSummary {
    pub id: Uuid,
    pub title: String,
    pub indent_level: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivePageItemCounts {
    pub outlines: usize,
    pub ink_strokes: usize,
    pub attachments: usize,
    pub printouts: usize,
    pub sheets: usize,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    jsonrpc: Option<String>,
    id: Option<serde_json::Value>,
    method: Option<String>,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    fn validate_version(&self) -> Result<(), AutomationError> {
        match self.jsonrpc.as_deref() {
            Some("2.0") => Ok(()),
            Some(other) => Err(AutomationError::Rpc(format!(
                "Invalid JSON-RPC version: {}",
                other
            ))),
            None => Err(AutomationError::Rpc(
                "Missing JSON-RPC 2.0 version".to_string(),
            )),
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonRpcSuccessResponse<'a> {
    jsonrpc: &'static str,
    id: &'a serde_json::Value,
    result: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorDetails {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorResponse<'a> {
    jsonrpc: &'static str,
    id: &'a serde_json::Value,
    error: JsonRpcErrorDetails,
}

pub fn inspect_session(session: &EngineSession) -> Result<NotebookSummary, AutomationError> {
    let notebook: &Notebook = &session.notebook;
    let active_section = session.active_section()?;

    let pages_summary = active_section
        .pages
        .iter()
        .map(|p| PageSummary {
            id: p.id,
            title: p.title.clone(),
            indent_level: p.indent_level,
        })
        .collect();

    let active_page = session.active_page()?;
    let item_counts = ActivePageItemCounts {
        outlines: active_page.outlines.len(),
        ink_strokes: active_page.ink_strokes.len(),
        attachments: active_page.attachments.len(),
        printouts: active_page.printouts.len(),
        sheets: active_page.sheets.len(),
    };

    Ok(NotebookSummary {
        id: notebook.id,
        name: notebook.name.clone(),
        active_section_id: session.active_section_id,
        active_page_id: session.active_page_id,
        sections_count: notebook.sections.len(),
        pages: pages_summary,
        active_page_item_counts: item_counts,
    })
}

pub fn handle_jsonrpc_on_session(
    session: &mut EngineSession,
    request_json: &str,
) -> Option<String> {
    if request_json.trim().is_empty() {
        return None;
    }

    let req: JsonRpcRequest = match serde_json::from_str(request_json) {
        Ok(r) => r,
        Err(_) => {
            let err_resp = JsonRpcErrorResponse {
                jsonrpc: "2.0",
                id: &serde_json::Value::Null,
                error: JsonRpcErrorDetails {
                    code: -32700,
                    message: "Parse error".to_string(),
                },
            };
            return serde_json::to_string(&err_resp).ok();
        }
    };

    let req_id = match &req.id {
        Some(id) if !id.is_null() => id,
        _ => return None, // Notification or missing ID
    };

    if let Err(err) = req.validate_version() {
        let err_resp = JsonRpcErrorResponse {
            jsonrpc: "2.0",
            id: req_id,
            error: JsonRpcErrorDetails {
                code: -32600,
                message: err.to_string(),
            },
        };
        return serde_json::to_string(&err_resp).ok();
    }

    let method = match &req.method {
        Some(m) => m.as_str(),
        None => {
            let err_resp = JsonRpcErrorResponse {
                jsonrpc: "2.0",
                id: req_id,
                error: JsonRpcErrorDetails {
                    code: -32601,
                    message: "Method not found".to_string(),
                },
            };
            return serde_json::to_string(&err_resp).ok();
        }
    };

    let response = match method {
        "initialize" => {
            let result = serde_json::json!({
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "quadernote-mcp",
                    "version": "0.1.0"
                }
            });
            serde_json::to_string(&JsonRpcSuccessResponse {
                jsonrpc: "2.0",
                id: req_id,
                result,
            })
        }
        "tools/list" => {
            let result = serde_json::json!({
                "tools": [
                    {
                        "name": "inspect_notebook",
                        "description": "Returns a structured JSON summary of the active Notebook",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "execute_command",
                        "description": "Executes a quader-engine Command and returns the CommandOutcome",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "cmd": { "type": "string" },
                                "args": { "type": "object" }
                            },
                            "required": ["cmd"]
                        }
                    },
                    {
                        "name": "search_notes",
                        "description": "Searches text across all pages in the notebook",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string" }
                            },
                            "required": ["query"]
                        }
                    }
                ]
            });
            serde_json::to_string(&JsonRpcSuccessResponse {
                jsonrpc: "2.0",
                id: req_id,
                result,
            })
        }
        "tools/call" => {
            let params = req.params.as_ref();
            let tool_name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str());
            let empty_args = serde_json::json!({});
            let arguments = params
                .and_then(|p| p.get("arguments"))
                .unwrap_or(&empty_args);

            match tool_name {
                Some("inspect_notebook") => match inspect_session(session) {
                    Ok(summary) => match serde_json::to_value(&summary) {
                        Ok(data) => {
                            let text = serde_json::to_string(&data).unwrap_or_default();
                            let result = serde_json::json!({
                                "content": [{ "type": "text", "text": text }],
                                "data": data,
                                "isError": false
                            });
                            serde_json::to_string(&JsonRpcSuccessResponse {
                                jsonrpc: "2.0",
                                id: req_id,
                                result,
                            })
                        }
                        Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                            jsonrpc: "2.0",
                            id: req_id,
                            error: JsonRpcErrorDetails {
                                code: -32000,
                                message: e.to_string(),
                            },
                        }),
                    },
                    Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                        jsonrpc: "2.0",
                        id: req_id,
                        error: JsonRpcErrorDetails {
                            code: -32000,
                            message: e.to_string(),
                        },
                    }),
                },
                Some("execute_command") => {
                    match serde_json::from_value::<Command>(arguments.clone()) {
                        Ok(cmd) => match session.execute(cmd) {
                            Ok(outcome) => match serde_json::to_value(&outcome) {
                                Ok(data) => {
                                    let text = serde_json::to_string(&data).unwrap_or_default();
                                    let result = serde_json::json!({
                                        "content": [{ "type": "text", "text": text }],
                                        "data": data,
                                        "isError": false
                                    });
                                    serde_json::to_string(&JsonRpcSuccessResponse {
                                        jsonrpc: "2.0",
                                        id: req_id,
                                        result,
                                    })
                                }
                                Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                                    jsonrpc: "2.0",
                                    id: req_id,
                                    error: JsonRpcErrorDetails {
                                        code: -32000,
                                        message: e.to_string(),
                                    },
                                }),
                            },
                            Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                                jsonrpc: "2.0",
                                id: req_id,
                                error: JsonRpcErrorDetails {
                                    code: -32000,
                                    message: e.to_string(),
                                },
                            }),
                        },
                        Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                            jsonrpc: "2.0",
                            id: req_id,
                            error: JsonRpcErrorDetails {
                                code: -32602,
                                message: format!("Invalid params for execute_command: {}", e),
                            },
                        }),
                    }
                }
                Some("search_notes") => {
                    let query = arguments.get("query").and_then(|q| q.as_str());
                    match query {
                        Some(q) => {
                            let hits: Vec<SearchHit> = session.search_text(q);
                            match serde_json::to_value(&hits) {
                                Ok(data) => {
                                    let text = serde_json::to_string(&data).unwrap_or_default();
                                    let result = serde_json::json!({
                                        "content": [{ "type": "text", "text": text }],
                                        "data": data,
                                        "isError": false
                                    });
                                    serde_json::to_string(&JsonRpcSuccessResponse {
                                        jsonrpc: "2.0",
                                        id: req_id,
                                        result,
                                    })
                                }
                                Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                                    jsonrpc: "2.0",
                                    id: req_id,
                                    error: JsonRpcErrorDetails {
                                        code: -32000,
                                        message: e.to_string(),
                                    },
                                }),
                            }
                        }
                        None => serde_json::to_string(&JsonRpcErrorResponse {
                            jsonrpc: "2.0",
                            id: req_id,
                            error: JsonRpcErrorDetails {
                                code: -32602,
                                message: "Missing 'query' argument".to_string(),
                            },
                        }),
                    }
                }
                _ => serde_json::to_string(&JsonRpcErrorResponse {
                    jsonrpc: "2.0",
                    id: req_id,
                    error: JsonRpcErrorDetails {
                        code: -32601,
                        message: "Unknown tool name".to_string(),
                    },
                }),
            }
        }
        "session/undo" => match session.undo() {
            Ok(()) => serde_json::to_string(&JsonRpcSuccessResponse {
                jsonrpc: "2.0",
                id: req_id,
                result: serde_json::json!({ "status": "ok" }),
            }),
            Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                jsonrpc: "2.0",
                id: req_id,
                error: JsonRpcErrorDetails {
                    code: -32000,
                    message: e.to_string(),
                },
            }),
        },
        "session/redo" => match session.redo() {
            Ok(()) => serde_json::to_string(&JsonRpcSuccessResponse {
                jsonrpc: "2.0",
                id: req_id,
                result: serde_json::json!({ "status": "ok" }),
            }),
            Err(e) => serde_json::to_string(&JsonRpcErrorResponse {
                jsonrpc: "2.0",
                id: req_id,
                error: JsonRpcErrorDetails {
                    code: -32000,
                    message: e.to_string(),
                },
            }),
        },
        _ => serde_json::to_string(&JsonRpcErrorResponse {
            jsonrpc: "2.0",
            id: req_id,
            error: JsonRpcErrorDetails {
                code: -32601,
                message: format!("Method not found: {}", method),
            },
        }),
    };

    response.ok()
}

pub struct AutomationServer {
    session: EngineSession,
}

impl AutomationServer {
    pub fn new(session: EngineSession) -> Self {
        Self { session }
    }

    pub fn session(&self) -> &EngineSession {
        &self.session
    }

    pub fn inspect_notebook(&self) -> Result<NotebookSummary, AutomationError> {
        inspect_session(&self.session)
    }

    pub fn handle_jsonrpc(&mut self, request_json: &str) -> Option<String> {
        handle_jsonrpc_on_session(&mut self.session, request_json)
    }

    pub fn run_batch_jsonl<R: BufRead>(
        &mut self,
        reader: R,
    ) -> Result<Vec<CommandOutcome>, AutomationError> {
        let mut outcomes = Vec::new();
        for line_res in reader.lines() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let cmd: Command = serde_json::from_str(trimmed)?;
            let outcome = self.session.execute(cmd)?;
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    pub fn serve_stdio<R: BufRead, W: Write>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> Result<(), AutomationError> {
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            if let Some(resp) = self.handle_jsonrpc(&line) {
                writer.write_all(resp.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
            line.clear();
        }
        Ok(())
    }

    pub fn serve_tcp_once(&mut self, listener: &TcpListener) -> Result<(), AutomationError> {
        let (stream, _) = listener.accept()?;
        let reader = BufReader::new(stream.try_clone()?);
        let writer = stream;
        self.serve_stdio(reader, writer)?;
        Ok(())
    }

    pub fn serve_tcp_loop(&mut self, addr: &str) -> Result<(), AutomationError> {
        let listener = TcpListener::bind(addr)?;
        loop {
            if let Err(e) = self.serve_tcp_once(&listener) {
                // If a connection drops with IO error or bad request, log or handle gracefully
                // but continue serving next TCP connection.
                let _ = e;
            }
        }
    }
}
