//! Headless CLI runner for QuaderNote.

use quader_automation::AutomationServer;
use quader_engine::EngineSession;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run_cli<I, W>(args: I, out: &mut W) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
    W: Write,
{
    let mut args_iter = args.into_iter();
    let _prog = args_iter.next();

    let subcommand = match args_iter.next() {
        Some(cmd) => cmd,
        None => {
            return Err(
                "Usage: quadernote-cli <inspect|mcp|batch <path>|serve [--port <u16>]>".to_string(),
            );
        }
    };

    match subcommand.as_str() {
        "inspect" => {
            let session = EngineSession::new("My Notebook");
            let server = AutomationServer::new(session);
            let summary = server
                .inspect_notebook()
                .map_err(|e| format!("Inspect error: {}", e))?;
            let json = serde_json::to_string_pretty(&summary)
                .map_err(|e| format!("Serialization error: {}", e))?;
            writeln!(out, "{}", json).map_err(|e| e.to_string())?;
        }
        "mcp" => {
            let session = EngineSession::new("My Notebook");
            let mut server = AutomationServer::new(session);
            let stdin = io::stdin();
            let stdout = io::stdout();
            server
                .serve_stdio(stdin.lock(), stdout.lock())
                .map_err(|e| format!("MCP stdio error: {}", e))?;
        }
        "batch" => {
            let path = match args_iter.next() {
                Some(p) => p,
                None => return Err("Usage: quadernote-cli batch <path>".to_string()),
            };

            let session = EngineSession::new("My Notebook");
            let mut server = AutomationServer::new(session);

            let reader: Box<dyn BufRead> = if path == "-" {
                Box::new(BufReader::new(io::stdin()))
            } else {
                let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
                Box::new(BufReader::new(file))
            };

            server
                .run_batch_jsonl(reader)
                .map_err(|e| format!("Batch error: {}", e))?;

            let notebook_json = serde_json::to_string_pretty(&server.session().notebook)
                .map_err(|e| format!("Serialization error: {}", e))?;
            writeln!(out, "{}", notebook_json).map_err(|e| e.to_string())?;
        }
        "serve" => {
            let mut port: u16 = 7982;
            while let Some(arg) = args_iter.next() {
                if arg == "--port" {
                    if let Some(p_str) = args_iter.next() {
                        port = p_str
                            .parse::<u16>()
                            .map_err(|_| format!("Invalid port number: {}", p_str))?;
                    } else {
                        return Err("Missing value for --port".to_string());
                    }
                }
            }

            let session = EngineSession::new("My Notebook");
            let mut server = AutomationServer::new(session);
            let addr = format!("127.0.0.1:{}", port);
            server
                .serve_tcp_loop(&addr)
                .map_err(|e| format!("TCP server error: {}", e))?;
        }
        other => {
            return Err(format!("Unknown subcommand: {}", other));
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut stdout = io::stdout();
    if let Err(err) = run_cli(args, &mut stdout) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpStream;
    use std::thread;

    #[test]
    fn test_cli_inspect() {
        let args = vec!["quadernote-cli".to_string(), "inspect".to_string()];
        let mut out = Vec::new();
        run_cli(args, &mut out).expect("cli inspect failed");
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("\"name\": \"My Notebook\""));
    }

    #[test]
    fn test_cli_batch_file() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let jsonl = "{\"cmd\": \"section.create\", \"args\": {\"name\": \"Batch Sec\", \"color_hex\": \"#123456\"}}\n";
        temp_file.write_all(jsonl.as_bytes()).unwrap();

        let args = vec![
            "quadernote-cli".to_string(),
            "batch".to_string(),
            temp_file.path().to_str().unwrap().to_string(),
        ];
        let mut out = Vec::new();
        run_cli(args, &mut out).expect("cli batch failed");
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Batch Sec"));
    }

    #[test]
    fn test_cli_serve() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // Free port for serve

        let args = vec![
            "quadernote-cli".to_string(),
            "serve".to_string(),
            "--port".to_string(),
            port.to_string(),
        ];

        let _handle = thread::spawn(move || {
            let mut out = Vec::new();
            let _ = run_cli(args, &mut out);
        });

        // Give server time to bind
        thread::sleep(std::time::Duration::from_millis(100));

        let stream = TcpStream::connect(format!("127.0.0.1:{}", port));
        assert!(stream.is_ok());
    }
}
