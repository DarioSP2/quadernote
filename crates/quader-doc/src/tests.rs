use crate::{InkSample, InkStroke, Notebook, Outline, PaperRuleStyle, Paragraph};
use quader_geom::{PointMm, RectMm};

#[test]
fn test_notebook_new() {
    let notebook = Notebook::new("My Test Notebook");
    assert_eq!(notebook.name, "My Test Notebook");
    assert_eq!(notebook.sections.len(), 1);

    let section = &notebook.sections[0];
    assert_eq!(section.name, "Quick Notes");
    assert_eq!(section.color_hex, "#7719AA");
    assert_eq!(section.pages.len(), 1);

    let page = &section.pages[0];
    assert_eq!(page.title, "Untitled Page");
    assert_eq!(page.indent_level, 0);
    assert_eq!(page.rule_style, PaperRuleStyle::None);
}

#[test]
fn test_ink_stroke_bounding_box() {
    let samples = vec![
        InkSample::new(PointMm::new(10.0, 10.0), 0.5),
        InkSample::new(PointMm::new(20.0, 30.0), 1.0),
    ];
    let stroke = InkStroke::new([0, 0, 0, 255], 2.0, false, samples);
    let bbox = stroke.bounding_box().expect("Bounding box should exist");

    // Base rect is (10, 10) to (20, 30), width 10, height 20.
    // Inflated by base_thickness_mm * 0.5 = 1.0 mm.
    assert_eq!(bbox, RectMm::new(9.0, 9.0, 12.0, 22.0));
}

#[test]
fn test_serde_json_roundtrip() {
    let mut notebook = Notebook::new("Serialized Notebook");
    let page = &mut notebook.sections[0].pages[0];

    let paragraph = Paragraph::new("Hello QuaderNote");
    let outline = Outline::new(RectMm::new(10.0, 10.0, 50.0, 20.0), vec![paragraph]);
    page.outlines.push(outline);

    let json = serde_json::to_string_pretty(&notebook).expect("Serialization failed");
    let deserialized: Notebook = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(notebook, deserialized);
}
