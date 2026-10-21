use quader_engine::Command;
use quader_ui_egui::QuaderUiState;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let output_path = if let Some(arg) = args.get(1) {
        PathBuf::from(arg)
    } else {
        PathBuf::from("target/shot.ppm")
    };

    let mut state = QuaderUiState::new();

    let _ = state.session.execute(Command::SectionCreate {
        name: "Research & Design".to_string(),
        color_hex: "#3B82F6".to_string(),
    });

    let _ = state.session.execute(Command::PageCreate {
        title: "Canvas Architecture".to_string(),
        indent_level: 0,
    });

    let _ = state.session.execute(Command::CanvasInsertOutline {
        x_mm: 20.0,
        y_mm: 20.0,
        width_mm: 120.0,
        initial_text: "QuaderNote 2D Spatial Canvas".to_string(),
    });

    let _ = state.session.execute(Command::InkAddStroke {
        color_rgba: [220, 38, 38, 255],
        thickness_mm: 1.0,
        is_highlighter: false,
        points: vec![(20.0, 40.0, 1.0), (80.0, 40.0, 1.0), (100.0, 60.0, 1.0)],
    });

    state.save_offscreen_shot(&output_path, 800, 600)?;
    println!("Saved offscreen screenshot to {:?}", output_path);

    Ok(())
}
