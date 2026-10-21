use super::*;
use quader_doc::PaperRuleStyle;
use quader_engine::Command;

#[test]
fn test_render_ui_tabs_and_rule_styles() {
    let ctx = egui::Context::default();
    let mut state = QuaderUiState::new();

    let tabs = [
        RibbonTab::Home,
        RibbonTab::Insert,
        RibbonTab::Draw,
        RibbonTab::View,
    ];

    let rule_styles = [
        PaperRuleStyle::None,
        PaperRuleStyle::RuledLines { spacing_mm: 8 },
        PaperRuleStyle::Grid { cell_mm: 5 },
        PaperRuleStyle::A4Guide210Mm,
    ];

    for tab in tabs {
        state.active_ribbon_tab = tab;
        for rule_style in rule_styles {
            let _ = state
                .session
                .execute(Command::PageSetRuleStyle { style: rule_style });
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                state.render_ui(ctx);
            });
        }
    }
}

#[test]
fn test_render_offscreen_ppm() {
    let mut state = QuaderUiState::new();

    let _ = state.session.execute(Command::CanvasInsertOutline {
        x_mm: 10.0,
        y_mm: 10.0,
        width_mm: 80.0,
        initial_text: "Test Outline".to_string(),
    });

    let _ = state.session.execute(Command::InkAddStroke {
        color_rgba: [255, 0, 0, 255],
        thickness_mm: 1.0,
        is_highlighter: false,
        points: vec![(10.0, 10.0, 1.0), (20.0, 20.0, 1.0)],
    });

    let ppm = state.render_offscreen_ppm(400, 300);
    assert!(ppm.starts_with(b"P6\n400 300\n255\n"));
    assert!(ppm.len() > 15);
}

#[test]
fn test_interactive_tools() {
    let mut state = QuaderUiState::new();
    state.active_tool = CanvasTool::Pen;
    state.current_stroke_points = vec![(5.0, 5.0, 1.0), (10.0, 10.0, 1.0)];

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        state.render_ui(ctx);
    });
}
