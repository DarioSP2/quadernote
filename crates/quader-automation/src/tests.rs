#[cfg(test)]
use crate::AutomationServer;
#[cfg(test)]
use quader_doc::PaperRuleStyle;
#[cfg(test)]
use quader_engine::{CommandOutcome, EngineSession};
#[cfg(test)]
use std::io::{BufReader, Cursor, Write};
#[cfg(test)]
use std::net::{TcpListener, TcpStream};
#[cfg(test)]
use std::thread;

#[test]
fn test_inspect_notebook() {
    let session = EngineSession::new("Test Notebook");
    let server = AutomationServer::new(session);
    let summary = server.inspect_notebook().expect("inspect_notebook failed");

    assert_eq!(summary.name, "Test Notebook");
    assert_eq!(summary.sections_count, 1);
    assert_eq!(summary.pages.len(), 1);
    assert_eq!(summary.active_page_item_counts.outlines, 0);
    assert_eq!(summary.active_page_item_counts.ink_strokes, 0);
}

#[test]
fn test_mcp_initialize_and_tools_list() {
    let session = EngineSession::new("Test Notebook");
    let mut server = AutomationServer::new(session);

    let init_req = r#"{"jsonrpc": "2.0", "id": 1, "method": "initialize"}"#;
    let init_resp = server
        .handle_jsonrpc(init_req)
        .expect("init response missing");
    let v: serde_json::Value = serde_json::from_str(&init_resp).unwrap();
    assert_eq!(v["id"], 1);
    assert_eq!(v["result"]["serverInfo"]["name"], "quadernote-mcp");

    let list_req = r#"{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}"#;
    let list_resp = server
        .handle_jsonrpc(list_req)
        .expect("list response missing");
    let v_list: serde_json::Value = serde_json::from_str(&list_resp).unwrap();
    assert_eq!(v_list["id"], 2);
    let tools = v_list["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 3);
}

#[test]
fn test_mcp_tools_call_inspect_notebook() {
    let session = EngineSession::new("Inspect Notebook Test");
    let mut server = AutomationServer::new(session);

    let call_req = r#"{"jsonrpc": "2.0", "id": 10, "method": "tools/call", "params": {"name": "inspect_notebook"}}"#;
    let resp = server.handle_jsonrpc(call_req).expect("call resp missing");
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["id"], 10);
    assert_eq!(v["result"]["data"]["name"], "Inspect Notebook Test");
    assert_eq!(v["result"]["isError"], false);
}

#[test]
fn test_mcp_tools_call_execute_command_and_undo_redo() {
    let session = EngineSession::new("Command Test");
    let mut server = AutomationServer::new(session);

    let call_cmd = "{\"jsonrpc\": \"2.0\", \"id\": 11, \"method\": \"tools/call\", \"params\": {\"name\": \"execute_command\", \"arguments\": {\"cmd\": \"section.create\", \"args\": {\"name\": \"New Section\", \"color_hex\": \"#FF0000\"}}}}";

    let resp = server.handle_jsonrpc(call_cmd).expect("cmd resp missing");
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["id"], 11);
    assert!(v["result"]["data"]["CreatedSection"].is_string());
    assert_eq!(server.session().notebook.sections.len(), 2);

    // Undo
    let undo_req = r#"{"jsonrpc": "2.0", "id": 12, "method": "session/undo"}"#;
    let undo_resp = server.handle_jsonrpc(undo_req).expect("undo resp missing");
    let v_undo: serde_json::Value = serde_json::from_str(&undo_resp).unwrap();
    assert_eq!(v_undo["result"]["status"], "ok");
    assert_eq!(server.session().notebook.sections.len(), 1);

    // Redo
    let redo_req = r#"{"jsonrpc": "2.0", "id": 13, "method": "session/redo"}"#;
    let redo_resp = server.handle_jsonrpc(redo_req).expect("redo resp missing");
    let v_redo: serde_json::Value = serde_json::from_str(&redo_resp).unwrap();
    assert_eq!(v_redo["result"]["status"], "ok");
    assert_eq!(server.session().notebook.sections.len(), 2);
}

#[test]
fn test_mcp_tools_call_search_notes() {
    let session = EngineSession::new("Search Test");
    let mut server = AutomationServer::new(session);

    let call_insert = r#"{"jsonrpc": "2.0", "id": 20, "method": "tools/call", "params": {"name": "execute_command", "arguments": {"cmd": "canvas.insert_outline", "args": {"x_mm": 10.0, "y_mm": 20.0, "width_mm": 100.0, "initial_text": "Meeting notes regarding Project Alpha"}}}}"#;
    server.handle_jsonrpc(call_insert);

    let search_req = r#"{"jsonrpc": "2.0", "id": 21, "method": "tools/call", "params": {"name": "search_notes", "arguments": {"query": "Project Alpha"}}}"#;
    let resp = server.handle_jsonrpc(search_req).expect("search resp");
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["id"], 21);
    let hits = v["result"]["data"].as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["snippet"], "Meeting notes regarding Project Alpha");
}

#[test]
fn test_run_batch_jsonl() {
    let session = EngineSession::new("Batch Test");
    let mut server = AutomationServer::new(session);

    let jsonl = "{\"cmd\": \"section.create\", \"args\": {\"name\": \"Sec 1\", \"color_hex\": \"#000000\"}}\n{\"cmd\": \"page.create\", \"args\": {\"title\": \"Page 1\", \"indent_level\": 0}}\n{\"cmd\": \"page.set_rule_style\", \"args\": {\"style\": {\"Grid\": {\"cell_mm\": 5}}}}\n";

    let outcomes = server
        .run_batch_jsonl(Cursor::new(jsonl))
        .expect("batch jsonl failed");
    assert_eq!(outcomes.len(), 3);
    assert!(matches!(outcomes[0], CommandOutcome::CreatedSection(_)));
    assert!(matches!(outcomes[1], CommandOutcome::CreatedPage(_)));
    assert_eq!(outcomes[2], CommandOutcome::Success);

    let active_page = server.session().active_page().unwrap();
    assert_eq!(active_page.rule_style, PaperRuleStyle::Grid { cell_mm: 5 });
}

#[test]
fn test_serve_stdio() {
    let session = EngineSession::new("Stdio Test");
    let mut server = AutomationServer::new(session);

    let input = "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"initialize\"}\n";
    let mut output = Vec::new();

    server
        .serve_stdio(Cursor::new(input), &mut output)
        .expect("serve_stdio failed");

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("quadernote-mcp"));
}

#[test]
fn test_tcp_round_trip() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().unwrap();

    let handle = thread::spawn(move || {
        let session = EngineSession::new("TCP Test");
        let mut server = AutomationServer::new(session);
        server.serve_tcp_once(&listener).expect("tcp serve failed");
    });

    let mut client_stream = TcpStream::connect(addr).expect("tcp connect");
    let req = "{\"jsonrpc\": \"2.0\", \"id\": 99, \"method\": \"initialize\"}\n";
    client_stream
        .write_all(req.as_bytes())
        .expect("client write");
    client_stream.flush().expect("client flush");

    let reader = BufReader::new(client_stream.try_clone().unwrap());
    use std::io::BufRead;
    let mut lines = reader.lines();
    let resp_line = lines
        .next()
        .expect("expected line")
        .expect("line read error");

    let v: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
    assert_eq!(v["id"], 99);
    assert_eq!(v["result"]["serverInfo"]["name"], "quadernote-mcp");

    // Shutdown write side or drop client stream to send EOF to server
    client_stream
        .shutdown(std::net::Shutdown::Both)
        .expect("shutdown failed");

    handle.join().unwrap();
}
