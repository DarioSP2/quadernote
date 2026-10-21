use quader_doc::{
    CanvasAttachment, EmbeddedSheet, InkSample, InkStroke, NoteTag, Notebook, Outline, Page,
    PaperRuleStyle, Paragraph, PrintoutPage, PrintoutSourceKind, PrintoutStack, Section,
};
use quader_geom::{PointMm, RectMm};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[cfg(test)]
mod tests;

#[derive(Error, Debug, PartialEq)]
pub enum EngineError {
    #[error("Section not found: {0}")]
    SectionNotFound(Uuid),
    #[error("Page not found: {0}")]
    PageNotFound(Uuid),
    #[error("Outline not found: {0}")]
    OutlineNotFound(Uuid),
    #[error("Paragraph index {index} out of bounds for outline {outline_id}")]
    ParagraphIndexOutOfBounds { outline_id: Uuid, index: usize },
    #[error("Attachment not found: {0}")]
    AttachmentNotFound(Uuid),
    #[error("Printout not found: {0}")]
    PrintoutNotFound(Uuid),
    #[error("Sheet not found: {0}")]
    SheetNotFound(Uuid),
    #[error("Worksheet not found: {0}")]
    WorksheetNotFound(String),
    #[error("Nothing to undo")]
    CannotUndo,
    #[error("Nothing to redo")]
    CannotRedo,
    #[error("JSON command error: {0}")]
    JsonError(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandOutcome {
    Success,
    CreatedSection(Uuid),
    CreatedPage(Uuid),
    CreatedOutline(Uuid),
    AddedStroke(Uuid),
    InsertedAttachment(Uuid),
    InsertedPrintout(Uuid),
    InsertedSheet(Uuid),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum Command {
    #[serde(rename = "section.create")]
    SectionCreate { name: String, color_hex: String },
    #[serde(rename = "section.rename")]
    SectionRename { section_id: Uuid, new_name: String },
    #[serde(rename = "section.select")]
    SectionSelect { section_id: Uuid },

    #[serde(rename = "page.create")]
    PageCreate { title: String, indent_level: u8 },
    #[serde(rename = "page.rename")]
    PageRename { page_id: Uuid, new_title: String },
    #[serde(rename = "page.set_indent")]
    PageSetIndent { page_id: Uuid, indent_level: u8 },
    #[serde(rename = "page.set_rule_style")]
    PageSetRuleStyle { style: PaperRuleStyle },
    #[serde(rename = "page.select")]
    PageSelect { page_id: Uuid },

    #[serde(rename = "canvas.insert_outline")]
    CanvasInsertOutline {
        x_mm: f32,
        y_mm: f32,
        width_mm: f32,
        initial_text: String,
    },
    #[serde(rename = "outline.move")]
    OutlineMove {
        outline_id: Uuid,
        new_origin_mm: PointMm,
    },
    #[serde(rename = "outline.append_paragraph")]
    OutlineAppendParagraph {
        outline_id: Uuid,
        text: String,
        tag: Option<NoteTag>,
    },

    #[serde(rename = "tag.set")]
    TagSet {
        outline_id: Uuid,
        paragraph_index: usize,
        tag: Option<NoteTag>,
    },
    #[serde(rename = "tag.toggle_todo")]
    TagToggleTodo {
        outline_id: Uuid,
        paragraph_index: usize,
    },

    #[serde(rename = "ink.add_stroke")]
    InkAddStroke {
        color_rgba: [u8; 4],
        thickness_mm: f32,
        is_highlighter: bool,
        points: Vec<(f32, f32, f32)>, // (x, y, pressure)
    },
    #[serde(rename = "ink.erase_stroke")]
    InkEraseStroke {
        x_mm: f32,
        y_mm: f32,
        eraser_radius_mm: f32,
    },

    #[serde(rename = "attachment.insert")]
    AttachmentInsert {
        file_name: String,
        mime_type: String,
        size_bytes: u64,
        stored_rel_path: String,
        sha256_hex: String,
        x_mm: f32,
        y_mm: f32,
    },
    #[serde(rename = "attachment.remove")]
    AttachmentRemove { attachment_id: Uuid },

    #[serde(rename = "printout.insert_pdf")]
    PrintoutInsertPdf {
        attachment_id: Uuid,
        x_mm: f32,
        y_mm: f32,
        page_width_mm: f32,
        page_height_mm: f32,
        gap_mm: f32,
        lock_pages: bool,
        page_texts: Vec<String>,
    },
    #[serde(rename = "printout.insert_pptx")]
    PrintoutInsertPptx {
        attachment_id: Uuid,
        x_mm: f32,
        y_mm: f32,
        slide_width_mm: f32,
        slide_height_mm: f32,
        gap_mm: f32,
        lock_slides: bool,
        slide_texts: Vec<String>,
    },
    #[serde(rename = "printout.set_lock")]
    PrintoutSetLock { printout_id: Uuid, locked: bool },

    #[serde(rename = "sheet.insert_xlsx")]
    SheetInsertXlsx {
        attachment_id: Uuid,
        x_mm: f32,
        y_mm: f32,
        width_mm: f32,
        height_mm: f32,
        worksheet_names: Vec<String>,
        active_worksheet: String,
        visible_range: Option<String>,
        cells: Vec<Vec<String>>,
    },
    #[serde(rename = "sheet.select_tab")]
    SheetSelectTab { sheet_id: Uuid, worksheet: String },
    #[serde(rename = "sheet.refresh")]
    SheetRefresh {
        sheet_id: Uuid,
        cells: Vec<Vec<String>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    pub section_id: Uuid,
    pub page_id: Uuid,
    pub page_title: String,
    pub snippet: String,
}

pub struct EngineSession {
    pub notebook: Notebook,
    pub active_section_id: Uuid,
    pub active_page_id: Uuid,
    undo_stack: Vec<(Notebook, Uuid, Uuid)>,
    redo_stack: Vec<(Notebook, Uuid, Uuid)>,
}

impl EngineSession {
    pub fn new(notebook_name: impl Into<String>) -> Self {
        let notebook = Notebook::new(notebook_name);
        let active_section_id = notebook
            .sections
            .first()
            .map(|s| s.id)
            .unwrap_or_else(Uuid::new_v4);
        let active_page_id = notebook
            .sections
            .first()
            .and_then(|s| s.pages.first())
            .map(|p| p.id)
            .unwrap_or_else(Uuid::new_v4);

        Self {
            notebook,
            active_section_id,
            active_page_id,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn active_section(&self) -> Result<&Section, EngineError> {
        self.notebook
            .sections
            .iter()
            .find(|s| s.id == self.active_section_id)
            .ok_or(EngineError::SectionNotFound(self.active_section_id))
    }

    pub fn active_page(&self) -> Result<&Page, EngineError> {
        let section = self.active_section()?;
        section
            .pages
            .iter()
            .find(|p| p.id == self.active_page_id)
            .ok_or(EngineError::PageNotFound(self.active_page_id))
    }

    fn active_page_mut(&mut self) -> Result<&mut Page, EngineError> {
        let sec_id = self.active_section_id;
        let page_id = self.active_page_id;

        let section = self
            .notebook
            .sections
            .iter_mut()
            .find(|s| s.id == sec_id)
            .ok_or(EngineError::SectionNotFound(sec_id))?;

        section
            .pages
            .iter_mut()
            .find(|p| p.id == page_id)
            .ok_or(EngineError::PageNotFound(page_id))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> Result<(), EngineError> {
        let (prev_notebook, prev_sec, prev_page) =
            self.undo_stack.pop().ok_or(EngineError::CannotUndo)?;
        let current_state = (
            self.notebook.clone(),
            self.active_section_id,
            self.active_page_id,
        );
        self.redo_stack.push(current_state);

        self.notebook = prev_notebook;
        self.active_section_id = prev_sec;
        self.active_page_id = prev_page;
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), EngineError> {
        let (next_notebook, next_sec, next_page) =
            self.redo_stack.pop().ok_or(EngineError::CannotRedo)?;
        let current_state = (
            self.notebook.clone(),
            self.active_section_id,
            self.active_page_id,
        );
        self.undo_stack.push(current_state);

        self.notebook = next_notebook;
        self.active_section_id = next_sec;
        self.active_page_id = next_page;
        Ok(())
    }

    pub fn execute_json(&mut self, json: &str) -> Result<CommandOutcome, EngineError> {
        let cmd: Command =
            serde_json::from_str(json).map_err(|e| EngineError::JsonError(e.to_string()))?;
        self.execute(cmd)
    }

    pub fn execute(&mut self, cmd: Command) -> Result<CommandOutcome, EngineError> {
        // Snapshot for undo history
        let snapshot = (
            self.notebook.clone(),
            self.active_section_id,
            self.active_page_id,
        );

        let outcome = match cmd {
            Command::SectionCreate { name, color_hex } => {
                let section = Section::new(name, color_hex);
                let id = section.id;
                self.notebook.sections.push(section);
                self.active_section_id = id;
                CommandOutcome::CreatedSection(id)
            }
            Command::SectionRename {
                section_id,
                new_name,
            } => {
                let section = self
                    .notebook
                    .sections
                    .iter_mut()
                    .find(|s| s.id == section_id)
                    .ok_or(EngineError::SectionNotFound(section_id))?;
                section.name = new_name;
                CommandOutcome::Success
            }
            Command::SectionSelect { section_id } => {
                let section = self
                    .notebook
                    .sections
                    .iter()
                    .find(|s| s.id == section_id)
                    .ok_or(EngineError::SectionNotFound(section_id))?;
                self.active_section_id = section.id;
                if let Some(first_page) = section.pages.first() {
                    self.active_page_id = first_page.id;
                }
                CommandOutcome::Success
            }
            Command::PageCreate {
                title,
                indent_level,
            } => {
                let sec_id = self.active_section_id;
                let section = self
                    .notebook
                    .sections
                    .iter_mut()
                    .find(|s| s.id == sec_id)
                    .ok_or(EngineError::SectionNotFound(sec_id))?;

                let mut page = Page::new(title);
                page.indent_level = indent_level.clamp(0, 2);
                let page_id = page.id;
                section.pages.push(page);
                self.active_page_id = page_id;
                CommandOutcome::CreatedPage(page_id)
            }
            Command::PageRename { page_id, new_title } => {
                let mut found = false;
                for sec in &mut self.notebook.sections {
                    if let Some(page) = sec.pages.iter_mut().find(|p| p.id == page_id) {
                        page.title = new_title.clone();
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(EngineError::PageNotFound(page_id));
                }
                CommandOutcome::Success
            }
            Command::PageSetIndent {
                page_id,
                indent_level,
            } => {
                let mut found = false;
                for sec in &mut self.notebook.sections {
                    if let Some(page) = sec.pages.iter_mut().find(|p| p.id == page_id) {
                        page.indent_level = indent_level.clamp(0, 2);
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(EngineError::PageNotFound(page_id));
                }
                CommandOutcome::Success
            }
            Command::PageSetRuleStyle { style } => {
                let page = self.active_page_mut()?;
                page.rule_style = style;
                CommandOutcome::Success
            }
            Command::PageSelect { page_id } => {
                let mut found_sec = None;
                for sec in &self.notebook.sections {
                    if sec.pages.iter().any(|p| p.id == page_id) {
                        found_sec = Some(sec.id);
                        break;
                    }
                }
                if let Some(sec_id) = found_sec {
                    self.active_section_id = sec_id;
                    self.active_page_id = page_id;
                    CommandOutcome::Success
                } else {
                    return Err(EngineError::PageNotFound(page_id));
                }
            }
            Command::CanvasInsertOutline {
                x_mm,
                y_mm,
                width_mm,
                initial_text,
            } => {
                let page = self.active_page_mut()?;
                let paragraph = Paragraph::new(initial_text);
                let bounds = RectMm::new(x_mm, y_mm, width_mm, 20.0);
                let outline = Outline::new(bounds, vec![paragraph]);
                let id = outline.id;
                page.outlines.push(outline);
                CommandOutcome::CreatedOutline(id)
            }
            Command::OutlineMove {
                outline_id,
                new_origin_mm,
            } => {
                let page = self.active_page_mut()?;
                let outline = page
                    .outlines
                    .iter_mut()
                    .find(|o| o.id == outline_id)
                    .ok_or(EngineError::OutlineNotFound(outline_id))?;
                outline.bounds_mm.origin = new_origin_mm;
                CommandOutcome::Success
            }
            Command::OutlineAppendParagraph {
                outline_id,
                text,
                tag,
            } => {
                let page = self.active_page_mut()?;
                let outline = page
                    .outlines
                    .iter_mut()
                    .find(|o| o.id == outline_id)
                    .ok_or(EngineError::OutlineNotFound(outline_id))?;
                let mut para = Paragraph::new(text);
                para.tag = tag;
                outline.paragraphs.push(para);
                CommandOutcome::Success
            }
            Command::TagSet {
                outline_id,
                paragraph_index,
                tag,
            } => {
                let page = self.active_page_mut()?;
                let outline = page
                    .outlines
                    .iter_mut()
                    .find(|o| o.id == outline_id)
                    .ok_or(EngineError::OutlineNotFound(outline_id))?;
                let para = outline.paragraphs.get_mut(paragraph_index).ok_or(
                    EngineError::ParagraphIndexOutOfBounds {
                        outline_id,
                        index: paragraph_index,
                    },
                )?;
                para.tag = tag;
                CommandOutcome::Success
            }
            Command::TagToggleTodo {
                outline_id,
                paragraph_index,
            } => {
                let page = self.active_page_mut()?;
                let outline = page
                    .outlines
                    .iter_mut()
                    .find(|o| o.id == outline_id)
                    .ok_or(EngineError::OutlineNotFound(outline_id))?;
                let para = outline.paragraphs.get_mut(paragraph_index).ok_or(
                    EngineError::ParagraphIndexOutOfBounds {
                        outline_id,
                        index: paragraph_index,
                    },
                )?;
                match &para.tag {
                    Some(NoteTag::Todo { checked }) => {
                        para.tag = Some(NoteTag::Todo { checked: !checked });
                    }
                    _ => {
                        para.tag = Some(NoteTag::Todo { checked: true });
                    }
                }
                CommandOutcome::Success
            }
            Command::InkAddStroke {
                color_rgba,
                thickness_mm,
                is_highlighter,
                points,
            } => {
                let page = self.active_page_mut()?;
                let samples = points
                    .into_iter()
                    .map(|(x, y, p)| InkSample::new(PointMm::new(x, y), p))
                    .collect();
                let stroke = InkStroke::new(color_rgba, thickness_mm, is_highlighter, samples);
                let id = stroke.id;
                page.ink_strokes.push(stroke);
                CommandOutcome::AddedStroke(id)
            }
            Command::InkEraseStroke {
                x_mm,
                y_mm,
                eraser_radius_mm,
            } => {
                let page = self.active_page_mut()?;
                let target = PointMm::new(x_mm, y_mm);
                page.ink_strokes.retain(|stroke| {
                    let pts: Vec<PointMm> = stroke.points.iter().map(|s| s.pos_mm).collect();
                    !quader_geom::polyline_hit_test_mm(&pts, target, eraser_radius_mm)
                });
                CommandOutcome::Success
            }
            Command::AttachmentInsert {
                file_name,
                mime_type,
                size_bytes,
                stored_rel_path,
                sha256_hex,
                x_mm,
                y_mm,
            } => {
                let page = self.active_page_mut()?;
                let attachment = CanvasAttachment {
                    id: Uuid::new_v4(),
                    pos_mm: PointMm::new(x_mm, y_mm),
                    file_name,
                    mime_type,
                    size_bytes,
                    stored_rel_path,
                    sha256_hex,
                };
                let id = attachment.id;
                page.attachments.push(attachment);
                CommandOutcome::InsertedAttachment(id)
            }
            Command::AttachmentRemove { attachment_id } => {
                let page = self.active_page_mut()?;
                let len_before = page.attachments.len();
                page.attachments.retain(|a| a.id != attachment_id);
                if page.attachments.len() == len_before {
                    return Err(EngineError::AttachmentNotFound(attachment_id));
                }
                CommandOutcome::Success
            }
            Command::PrintoutInsertPdf {
                attachment_id,
                x_mm,
                y_mm,
                page_width_mm,
                page_height_mm,
                gap_mm,
                lock_pages,
                page_texts,
            } => {
                let page = self.active_page_mut()?;
                let mut printout_pages = Vec::new();
                for (idx, text) in page_texts.into_iter().enumerate() {
                    let page_y = y_mm + (idx as f32) * (page_height_mm + gap_mm);
                    printout_pages.push(PrintoutPage {
                        page_number: (idx + 1) as u32,
                        bounds_mm: RectMm::new(x_mm, page_y, page_width_mm, page_height_mm),
                        locked_to_background: lock_pages,
                        extracted_text: text,
                        rendered_image_rel_path: format!("printouts/{}_{}.png", attachment_id, idx),
                    });
                }
                let stack = PrintoutStack {
                    id: Uuid::new_v4(),
                    attachment_id,
                    kind: PrintoutSourceKind::Pdf,
                    pages: printout_pages,
                };
                let id = stack.id;
                page.printouts.push(stack);
                CommandOutcome::InsertedPrintout(id)
            }
            Command::PrintoutInsertPptx {
                attachment_id,
                x_mm,
                y_mm,
                slide_width_mm,
                slide_height_mm,
                gap_mm,
                lock_slides,
                slide_texts,
            } => {
                let page = self.active_page_mut()?;
                let mut printout_pages = Vec::new();
                for (idx, text) in slide_texts.into_iter().enumerate() {
                    let slide_y = y_mm + (idx as f32) * (slide_height_mm + gap_mm);
                    printout_pages.push(PrintoutPage {
                        page_number: (idx + 1) as u32,
                        bounds_mm: RectMm::new(x_mm, slide_y, slide_width_mm, slide_height_mm),
                        locked_to_background: lock_slides,
                        extracted_text: text,
                        rendered_image_rel_path: format!("printouts/{}_{}.png", attachment_id, idx),
                    });
                }
                let stack = PrintoutStack {
                    id: Uuid::new_v4(),
                    attachment_id,
                    kind: PrintoutSourceKind::PowerPointPptx,
                    pages: printout_pages,
                };
                let id = stack.id;
                page.printouts.push(stack);
                CommandOutcome::InsertedPrintout(id)
            }
            Command::PrintoutSetLock {
                printout_id,
                locked,
            } => {
                let page = self.active_page_mut()?;
                let printout = page
                    .printouts
                    .iter_mut()
                    .find(|p| p.id == printout_id)
                    .ok_or(EngineError::PrintoutNotFound(printout_id))?;
                for p in &mut printout.pages {
                    p.locked_to_background = locked;
                }
                CommandOutcome::Success
            }
            Command::SheetInsertXlsx {
                attachment_id,
                x_mm,
                y_mm,
                width_mm,
                height_mm,
                worksheet_names,
                active_worksheet,
                visible_range,
                cells,
            } => {
                let page = self.active_page_mut()?;
                let sheet = EmbeddedSheet {
                    id: Uuid::new_v4(),
                    attachment_id,
                    bounds_mm: RectMm::new(x_mm, y_mm, width_mm, height_mm),
                    worksheet_names,
                    active_worksheet,
                    visible_range,
                    cells,
                };
                let id = sheet.id;
                page.sheets.push(sheet);
                CommandOutcome::InsertedSheet(id)
            }
            Command::SheetSelectTab {
                sheet_id,
                worksheet,
            } => {
                let page = self.active_page_mut()?;
                let sheet = page
                    .sheets
                    .iter_mut()
                    .find(|s| s.id == sheet_id)
                    .ok_or(EngineError::SheetNotFound(sheet_id))?;
                if !sheet.worksheet_names.contains(&worksheet) {
                    return Err(EngineError::WorksheetNotFound(worksheet));
                }
                sheet.active_worksheet = worksheet;
                CommandOutcome::Success
            }
            Command::SheetRefresh { sheet_id, cells } => {
                let page = self.active_page_mut()?;
                let sheet = page
                    .sheets
                    .iter_mut()
                    .find(|s| s.id == sheet_id)
                    .ok_or(EngineError::SheetNotFound(sheet_id))?;
                sheet.cells = cells;
                CommandOutcome::Success
            }
        };

        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
        Ok(outcome)
    }

    pub fn search_text(&self, query: &str) -> Vec<SearchHit> {
        let q = query.to_lowercase();
        let mut results = Vec::new();

        for section in &self.notebook.sections {
            for page in &section.pages {
                let mut matched = false;
                let mut snippet = String::new();

                if page.title.to_lowercase().contains(&q) {
                    matched = true;
                    snippet = page.title.clone();
                }

                if !matched {
                    for outline in &page.outlines {
                        for para in &outline.paragraphs {
                            if para.text.to_lowercase().contains(&q) {
                                matched = true;
                                snippet = para.text.clone();
                                break;
                            }
                        }
                        if matched {
                            break;
                        }
                    }
                }

                if !matched {
                    for att in &page.attachments {
                        if att.file_name.to_lowercase().contains(&q) {
                            matched = true;
                            snippet = att.file_name.clone();
                            break;
                        }
                    }
                }

                if !matched {
                    for printout in &page.printouts {
                        for ppage in &printout.pages {
                            if ppage.extracted_text.to_lowercase().contains(&q) {
                                matched = true;
                                snippet = ppage.extracted_text.clone();
                                break;
                            }
                        }
                        if matched {
                            break;
                        }
                    }
                }

                if !matched {
                    for sheet in &page.sheets {
                        for row in &sheet.cells {
                            for cell in row {
                                if cell.to_lowercase().contains(&q) {
                                    matched = true;
                                    snippet = cell.clone();
                                    break;
                                }
                            }
                            if matched {
                                break;
                            }
                        }
                        if matched {
                            break;
                        }
                    }
                }

                if matched {
                    results.push(SearchHit {
                        section_id: section.id,
                        page_id: page.id,
                        page_title: page.title.clone(),
                        snippet,
                    });
                }
            }
        }

        results
    }

    pub fn search_by_tag(&self, target_tag: &NoteTag) -> Vec<SearchHit> {
        let mut results = Vec::new();

        for section in &self.notebook.sections {
            for page in &section.pages {
                let mut matched = false;
                let mut snippet = String::new();

                for outline in &page.outlines {
                    for para in &outline.paragraphs {
                        if let Some(tag) = &para.tag {
                            if tag == target_tag {
                                matched = true;
                                snippet = para.text.clone();
                                break;
                            }
                        }
                    }
                    if matched {
                        break;
                    }
                }

                if matched {
                    results.push(SearchHit {
                        section_id: section.id,
                        page_id: page.id,
                        page_title: page.title.clone(),
                        snippet,
                    });
                }
            }
        }

        results
    }
}
