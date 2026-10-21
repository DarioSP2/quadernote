use crate::{Command, CommandOutcome, EngineSession};
use quader_doc::NoteTag;

#[test]
fn test_engine_session_initialization() {
    let session = EngineSession::new("Engine Notebook");
    assert_eq!(session.notebook.name, "Engine Notebook");
    assert!(session.active_section().is_ok());
    assert!(session.active_page().is_ok());
    assert!(!session.can_undo());
    assert!(!session.can_redo());
}

#[test]
fn test_command_undo_redo_roundtrip() {
    let mut session = EngineSession::new("Undo Redo Notebook");
    let initial_section_count = session.notebook.sections.len();

    let cmd = Command::SectionCreate {
        name: "Research".to_string(),
        color_hex: "#00FF00".to_string(),
    };

    let outcome = session.execute(cmd).expect("Execution should succeed");
    if let CommandOutcome::CreatedSection(sec_id) = outcome {
        assert_eq!(session.active_section_id, sec_id);
    } else {
        panic!("Expected CreatedSection outcome");
    }

    assert_eq!(session.notebook.sections.len(), initial_section_count + 1);
    assert!(session.can_undo());
    assert!(!session.can_redo());

    // Undo -> state equals initial
    session.undo().expect("Undo should succeed");
    assert_eq!(session.notebook.sections.len(), initial_section_count);
    assert!(!session.can_undo());
    assert!(session.can_redo());

    // Redo -> state equals post-execute
    session.redo().expect("Redo should succeed");
    assert_eq!(session.notebook.sections.len(), initial_section_count + 1);
    assert!(session.can_undo());
    assert!(!session.can_redo());
}

#[test]
fn test_canvas_outline_and_tag_commands() {
    let mut session = EngineSession::new("Outline Notebook");

    let insert_cmd = Command::CanvasInsertOutline {
        x_mm: 10.0,
        y_mm: 10.0,
        width_mm: 100.0,
        initial_text: "First Paragraph".to_string(),
    };

    let outcome = session.execute(insert_cmd).expect("Insert should succeed");
    let outline_id = match outcome {
        CommandOutcome::CreatedOutline(id) => id,
        _ => panic!("Expected CreatedOutline"),
    };

    let tag_cmd = Command::TagSet {
        outline_id,
        paragraph_index: 0,
        tag: Some(NoteTag::Todo { checked: false }),
    };
    session.execute(tag_cmd).expect("TagSet should succeed");

    let toggle_cmd = Command::TagToggleTodo {
        outline_id,
        paragraph_index: 0,
    };
    session.execute(toggle_cmd).expect("Toggle should succeed");

    let page = session.active_page().expect("Page should exist");
    let outline = &page.outlines[0];
    assert_eq!(
        outline.paragraphs[0].tag,
        Some(NoteTag::Todo { checked: true })
    );

    // Search tests
    let hits = session.search_text("Paragraph");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].snippet, "First Paragraph");

    let tag_hits = session.search_by_tag(&NoteTag::Todo { checked: true });
    assert_eq!(tag_hits.len(), 1);
}

#[test]
fn test_json_command_execution() {
    let mut session = EngineSession::new("JSON Session");
    let json_cmd = r#"{
        "cmd": "page.create",
        "args": {
            "title": "New JSON Page",
            "indent_level": 1
        }
    }"#;

    let outcome = session
        .execute_json(json_cmd)
        .expect("JSON command execution failed");
    match outcome {
        CommandOutcome::CreatedPage(_) => (),
        _ => panic!("Expected CreatedPage outcome"),
    }

    assert_eq!(session.active_page().unwrap().title, "New JSON Page");
}
