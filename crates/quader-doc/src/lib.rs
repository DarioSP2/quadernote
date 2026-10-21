use quader_geom::{PointMm, RectMm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Note tags for paragraphs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteTag {
    Todo { checked: bool },
    Important,
    Question,
    Definition,
    Custom { label: String, icon_id: String },
}

/// Paper background rule style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PaperRuleStyle {
    #[default]
    None,
    RuledLines {
        spacing_mm: u8,
    },
    Grid {
        cell_mm: u8,
    },
    A4Guide210Mm,
}

/// Single paragraph inside a 2D Outline container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub id: Uuid,
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub font_size_mm: f32,
    pub tag: Option<NoteTag>,
}

impl Paragraph {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.into(),
            bold: false,
            italic: false,
            font_size_mm: 3.5, // ~10pt equivalent
            tag: None,
        }
    }
}

/// Floating 2D container for text paragraphs on canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outline {
    pub id: Uuid,
    pub bounds_mm: RectMm,
    pub paragraphs: Vec<Paragraph>,
}

impl Outline {
    pub fn new(bounds_mm: RectMm, paragraphs: Vec<Paragraph>) -> Self {
        Self {
            id: Uuid::new_v4(),
            bounds_mm,
            paragraphs,
        }
    }
}

/// Sample point in an ink stroke with pressure data.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InkSample {
    pub pos_mm: PointMm,
    pub pressure: f32,
}

impl InkSample {
    pub fn new(pos_mm: PointMm, pressure: f32) -> Self {
        Self { pos_mm, pressure }
    }
}

/// Pressure-sensitive ink stroke spline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InkStroke {
    pub id: Uuid,
    pub color_rgba: [u8; 4],
    pub base_thickness_mm: f32,
    pub is_highlighter: bool,
    pub points: Vec<InkSample>,
}

impl InkStroke {
    pub fn new(
        color_rgba: [u8; 4],
        base_thickness_mm: f32,
        is_highlighter: bool,
        points: Vec<InkSample>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            color_rgba,
            base_thickness_mm,
            is_highlighter,
            points,
        }
    }

    pub fn bounding_box(&self) -> Option<RectMm> {
        let pts: Vec<PointMm> = self.points.iter().map(|s| s.pos_mm).collect();
        let bbox = RectMm::from_points(&pts)?;
        Some(bbox.inflate(self.base_thickness_mm * 0.5))
    }
}

/// Canvas attachment badge linking to locally stored file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasAttachment {
    pub id: Uuid,
    pub pos_mm: PointMm,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub stored_rel_path: String,
    pub sha256_hex: String,
}

/// Source kind for printout document stacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrintoutSourceKind {
    Pdf,
    PowerPointPptx,
}

/// Individual printout page image and text frame on canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintoutPage {
    pub page_number: u32,
    pub bounds_mm: RectMm,
    pub locked_to_background: bool,
    pub extracted_text: String,
    pub rendered_image_rel_path: String,
}

/// Stack of rendered document printouts (PDF/PPTX) on canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintoutStack {
    pub id: Uuid,
    pub attachment_id: Uuid,
    pub kind: PrintoutSourceKind,
    pub pages: Vec<PrintoutPage>,
}

/// Embedded spreadsheet grid widget on canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedSheet {
    pub id: Uuid,
    pub attachment_id: Uuid,
    pub bounds_mm: RectMm,
    pub worksheet_names: Vec<String>,
    pub active_worksheet: String,
    pub visible_range: Option<String>,
    pub cells: Vec<Vec<String>>,
}

/// Canvas Page container holding outlines, ink, attachments, printouts, and sheets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub title: String,
    pub indent_level: u8,
    pub created_at_iso: String,
    pub rule_style: PaperRuleStyle,
    pub outlines: Vec<Outline>,
    pub ink_strokes: Vec<InkStroke>,
    pub attachments: Vec<CanvasAttachment>,
    pub printouts: Vec<PrintoutStack>,
    pub sheets: Vec<EmbeddedSheet>,
}

impl Page {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            indent_level: 0,
            created_at_iso: "2026-01-01T00:00:00Z".to_string(),
            rule_style: PaperRuleStyle::None,
            outlines: Vec::new(),
            ink_strokes: Vec::new(),
            attachments: Vec::new(),
            printouts: Vec::new(),
            sheets: Vec::new(),
        }
    }
}

/// Section container holding pages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub id: Uuid,
    pub name: String,
    pub color_hex: String,
    pub pages: Vec<Page>,
}

impl Section {
    pub fn new(name: impl Into<String>, color_hex: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color_hex: color_hex.into(),
            pages: Vec::new(),
        }
    }
}

/// Top-level Notebook structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notebook {
    pub id: Uuid,
    pub name: String,
    pub sections: Vec<Section>,
}

impl Notebook {
    pub fn new(name: impl Into<String>) -> Self {
        let mut default_section = Section::new("Quick Notes", "#7719AA");
        default_section.pages.push(Page::new("Untitled Page"));

        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            sections: vec![default_section],
        }
    }
}
