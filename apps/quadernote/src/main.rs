use quader_automation::AutomationServer;
use quader_engine::EngineSession;
use quader_ui_egui::QuaderUiState;
use std::env;
use std::path::PathBuf;
use std::thread;

pub struct QuaderApp {
    pub state: QuaderUiState,
}

impl QuaderApp {
    pub fn new() -> Self {
        Self {
            state: QuaderUiState::new(),
        }
    }
}

impl Default for QuaderApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for QuaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.render_ui(ctx);
    }
}

pub struct CliArgs {
    pub shot_path: Option<PathBuf>,
    pub control_port: Option<u16>,
}

pub fn parse_cli_args<I, T>(args: I) -> CliArgs
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    let mut shot_path = None;
    let mut control_port = None;

    let args_vec: Vec<String> = args.into_iter().map(|a| a.into()).collect();
    let mut iter = args_vec.into_iter().skip(1);

    while let Some(arg) = iter.next() {
        if arg == "--shot" {
            if let Some(val) = iter.next() {
                shot_path = Some(PathBuf::from(val));
            }
        } else if arg == "--control" {
            if let Some(val) = iter.next() {
                if let Ok(port) = val.parse::<u16>() {
                    control_port = Some(port);
                }
            }
        }
    }

    CliArgs {
        shot_path,
        control_port,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = parse_cli_args(env::args());

    if let Some(shot_path) = cli_args.shot_path {
        let mut ui_state = QuaderUiState::new();
        ui_state.save_offscreen_shot(&shot_path, 800, 600)?;
        println!("Saved shot to {:?}", shot_path);
        return Ok(());
    }

    if let Some(port) = cli_args.control_port {
        let addr = format!("127.0.0.1:{}", port);
        thread::spawn(move || {
            let session = EngineSession::new("Control Session");
            let mut server = AutomationServer::new(session);
            println!("Control listener thread active on {}", addr);
            let _ = server.serve_tcp_loop(&addr);
        });
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("QuaderNote")
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "QuaderNote",
        native_options,
        Box::new(|_cc| Ok(Box::new(QuaderApp::new()))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_parsing() {
        let args = vec!["quadernote", "--shot", "output.ppm", "--control", "7982"];
        let parsed = parse_cli_args(args);
        assert_eq!(parsed.shot_path, Some(PathBuf::from("output.ppm")));
        assert_eq!(parsed.control_port, Some(7982));
    }

    #[test]
    fn test_headless_shot_execution() {
        let temp_dir = std::env::temp_dir();
        let shot_file = temp_dir.join("test_quadernote_shot.ppm");
        let mut ui_state = QuaderUiState::new();
        let res = ui_state.save_offscreen_shot(&shot_file, 400, 300);
        assert!(res.is_ok());
        assert!(shot_file.exists());
        let _ = std::fs::remove_file(shot_file);
    }
}
