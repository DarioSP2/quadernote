use quader_doc::{NoteTag, PaperRuleStyle};
use quader_engine::{Command, EngineSession, SearchHit};
use quader_geom::{PointMm, ViewportTransform};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PIXELS_PER_MM_BASE: f32 = 3.779_527_6;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RibbonTab {
    #[default]
    Home,
    Insert,
    Draw,
    View,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CanvasTool {
    #[default]
    Select,
    Pan,
    Pen,
    Highlighter,
    Eraser,
    InsertOutline,
}

pub struct QuaderUiState {
    pub session: EngineSession,
    pub viewport: ViewportTransform,
    pub active_ribbon_tab: RibbonTab,
    pub active_tool: CanvasTool,
    pub pen_color_rgba: [u8; 4],
    pub pen_thickness_mm: f32,
    pub highlighter_color_rgba: [u8; 4],
    pub highlighter_thickness_mm: f32,
    pub eraser_radius_mm: f32,
    pub search_query: String,
    pub last_search_hits: Vec<SearchHit>,
    pub current_stroke_points: Vec<(f32, f32, f32)>,
}

impl Default for QuaderUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl QuaderUiState {
    pub fn new() -> Self {
        let session = EngineSession::new("My Notebook");
        Self {
            session,
            viewport: ViewportTransform::default(),
            active_ribbon_tab: RibbonTab::Home,
            active_tool: CanvasTool::Select,
            pen_color_rgba: [20, 20, 20, 255],
            pen_thickness_mm: 0.5,
            highlighter_color_rgba: [255, 235, 59, 140],
            highlighter_thickness_mm: 4.0,
            eraser_radius_mm: 2.5,
            search_query: String::new(),
            last_search_hits: Vec::new(),
            current_stroke_points: Vec::new(),
        }
    }

    fn px_per_mm(&self) -> f32 {
        self.viewport.zoom * PIXELS_PER_MM_BASE
    }

    pub fn render_ui(&mut self, ctx: &egui::Context) {
        self.render_top_ribbon(ctx);
        self.render_left_section_rail(ctx);
        self.render_right_page_hierarchy(ctx);
        self.render_center_canvas(ctx);
    }

    fn render_top_ribbon(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_ribbon").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_ribbon_tab, RibbonTab::Home, "Home");
                    ui.selectable_value(&mut self.active_ribbon_tab, RibbonTab::Insert, "Insert");
                    ui.selectable_value(&mut self.active_ribbon_tab, RibbonTab::Draw, "Draw");
                    ui.selectable_value(&mut self.active_ribbon_tab, RibbonTab::View, "View");

                    ui.separator();

                    if ui
                        .add_enabled(self.session.can_undo(), egui::Button::new("Undo"))
                        .clicked()
                    {
                        let _ = self.session.undo();
                    }
                    if ui
                        .add_enabled(self.session.can_redo(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        let _ = self.session.redo();
                    }

                    ui.separator();

                    ui.label("Search:");
                    let search_response = ui.text_edit_singleline(&mut self.search_query);
                    if search_response.changed() {
                        if self.search_query.trim().is_empty() {
                            self.last_search_hits.clear();
                        } else {
                            self.last_search_hits = self.session.search_text(&self.search_query);
                        }
                    }
                });

                ui.separator();

                match self.active_ribbon_tab {
                    RibbonTab::Home => {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    self.active_tool == CanvasTool::Select,
                                    "Select Tool",
                                )
                                .clicked()
                            {
                                self.active_tool = CanvasTool::Select;
                            }
                            if ui
                                .selectable_label(
                                    self.active_tool == CanvasTool::InsertOutline,
                                    "Insert Outline Tool",
                                )
                                .clicked()
                            {
                                self.active_tool = CanvasTool::InsertOutline;
                            }
                            if ui.button("Add Sample Outline").clicked() {
                                let _ = self.session.execute(Command::CanvasInsertOutline {
                                    x_mm: 20.0,
                                    y_mm: 20.0,
                                    width_mm: 120.0,
                                    initial_text: "Sample Outline".to_string(),
                                });
                            }
                        });
                    }
                    RibbonTab::Insert => {
                        ui.horizontal(|ui| {
                            if ui.button("Insert Attachment").clicked() {
                                let _ = self.session.execute(Command::AttachmentInsert {
                                    file_name: "Document.pdf".to_string(),
                                    mime_type: "application/pdf".to_string(),
                                    size_bytes: 1024,
                                    stored_rel_path: "attachments/doc.pdf".to_string(),
                                    sha256_hex: "1234567890abcdef".to_string(),
                                    x_mm: 20.0,
                                    y_mm: 30.0,
                                });
                            }
                            if ui.button("Insert PDF Printout").clicked() {
                                let att_id = Uuid::new_v4();
                                let _ = self.session.execute(Command::AttachmentInsert {
                                    file_name: "Sample.pdf".to_string(),
                                    mime_type: "application/pdf".to_string(),
                                    size_bytes: 2048,
                                    stored_rel_path: "attachments/sample.pdf".to_string(),
                                    sha256_hex: "abcdef1234567890".to_string(),
                                    x_mm: 10.0,
                                    y_mm: 10.0,
                                });
                                let _ = self.session.execute(Command::PrintoutInsertPdf {
                                    attachment_id: att_id,
                                    x_mm: 20.0,
                                    y_mm: 50.0,
                                    page_width_mm: 190.0,
                                    page_height_mm: 270.0,
                                    gap_mm: 10.0,
                                    lock_pages: true,
                                    page_texts: vec![
                                        "PDF Page 1 content".to_string(),
                                        "PDF Page 2 content".to_string(),
                                    ],
                                });
                            }
                            if ui.button("Insert PPTX Printout").clicked() {
                                let att_id = Uuid::new_v4();
                                let _ = self.session.execute(Command::PrintoutInsertPptx {
                                    attachment_id: att_id,
                                    x_mm: 20.0,
                                    y_mm: 50.0,
                                    slide_width_mm: 190.0,
                                    slide_height_mm: 107.0,
                                    gap_mm: 10.0,
                                    lock_slides: true,
                                    slide_texts: vec![
                                        "Slide 1 Title".to_string(),
                                        "Slide 2 Title".to_string(),
                                    ],
                                });
                            }
                            if ui.button("Insert Excel Sheet").clicked() {
                                let att_id = Uuid::new_v4();
                                let _ = self.session.execute(Command::SheetInsertXlsx {
                                    attachment_id: att_id,
                                    x_mm: 20.0,
                                    y_mm: 50.0,
                                    width_mm: 150.0,
                                    height_mm: 80.0,
                                    worksheet_names: vec![
                                        "Sheet1".to_string(),
                                        "Sheet2".to_string(),
                                    ],
                                    active_worksheet: "Sheet1".to_string(),
                                    visible_range: Some("A1:C3".to_string()),
                                    cells: vec![
                                        vec![
                                            "Header A".to_string(),
                                            "Header B".to_string(),
                                            "Header C".to_string(),
                                        ],
                                        vec!["10".to_string(), "20".to_string(), "30".to_string()],
                                        vec!["40".to_string(), "50".to_string(), "60".to_string()],
                                    ],
                                });
                            }
                        });
                    }
                    RibbonTab::Draw => {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(self.active_tool == CanvasTool::Pan, "Pan")
                                .clicked()
                            {
                                self.active_tool = CanvasTool::Pan;
                            }
                            if ui
                                .selectable_label(self.active_tool == CanvasTool::Pen, "Pen")
                                .clicked()
                            {
                                self.active_tool = CanvasTool::Pen;
                            }
                            if ui
                                .selectable_label(
                                    self.active_tool == CanvasTool::Highlighter,
                                    "Highlighter",
                                )
                                .clicked()
                            {
                                self.active_tool = CanvasTool::Highlighter;
                            }
                            if ui
                                .selectable_label(self.active_tool == CanvasTool::Eraser, "Eraser")
                                .clicked()
                            {
                                self.active_tool = CanvasTool::Eraser;
                            }

                            ui.separator();

                            if self.active_tool == CanvasTool::Pen {
                                ui.label("Pen Thickness (mm):");
                                ui.add(
                                    egui::DragValue::new(&mut self.pen_thickness_mm)
                                        .range(0.1..=10.0)
                                        .speed(0.1),
                                );

                                if ui.button("Black").clicked() {
                                    self.pen_color_rgba = [20, 20, 20, 255];
                                }
                                if ui.button("Red").clicked() {
                                    self.pen_color_rgba = [220, 38, 38, 255];
                                }
                                if ui.button("Blue").clicked() {
                                    self.pen_color_rgba = [37, 99, 235, 255];
                                }
                            } else if self.active_tool == CanvasTool::Highlighter {
                                ui.label("Highlighter Thickness (mm):");
                                ui.add(
                                    egui::DragValue::new(&mut self.highlighter_thickness_mm)
                                        .range(1.0..=20.0)
                                        .speed(0.5),
                                );

                                if ui.button("Yellow").clicked() {
                                    self.highlighter_color_rgba = [255, 235, 59, 140];
                                }
                                if ui.button("Green").clicked() {
                                    self.highlighter_color_rgba = [76, 175, 80, 140];
                                }
                                if ui.button("Pink").clicked() {
                                    self.highlighter_color_rgba = [233, 30, 99, 140];
                                }
                            } else if self.active_tool == CanvasTool::Eraser {
                                ui.label("Eraser Radius (mm):");
                                ui.add(
                                    egui::DragValue::new(&mut self.eraser_radius_mm)
                                        .range(0.5..=20.0)
                                        .speed(0.5),
                                );
                            }
                        });
                    }
                    RibbonTab::View => {
                        ui.horizontal(|ui| {
                            ui.label("Rule Style:");
                            if ui.button("None").clicked() {
                                let _ = self.session.execute(Command::PageSetRuleStyle {
                                    style: PaperRuleStyle::None,
                                });
                            }
                            if ui.button("Ruled 8mm").clicked() {
                                let _ = self.session.execute(Command::PageSetRuleStyle {
                                    style: PaperRuleStyle::RuledLines { spacing_mm: 8 },
                                });
                            }
                            if ui.button("Grid 5mm").clicked() {
                                let _ = self.session.execute(Command::PageSetRuleStyle {
                                    style: PaperRuleStyle::Grid { cell_mm: 5 },
                                });
                            }
                            if ui.button("A4 Guide 210mm").clicked() {
                                let _ = self.session.execute(Command::PageSetRuleStyle {
                                    style: PaperRuleStyle::A4Guide210Mm,
                                });
                            }

                            ui.separator();

                            if ui.button("Zoom In").clicked() {
                                self.viewport.zoom_around_screen_point(
                                    self.viewport.zoom * 1.2,
                                    400.0,
                                    300.0,
                                );
                            }
                            if ui.button("Zoom Out").clicked() {
                                self.viewport.zoom_around_screen_point(
                                    self.viewport.zoom / 1.2,
                                    400.0,
                                    300.0,
                                );
                            }
                            if ui.button("Reset Zoom (100%)").clicked() {
                                self.viewport.zoom = 1.0;
                                self.viewport.pan_mm = PointMm::new(0.0, 0.0);
                            }
                        });
                    }
                }

                if !self.last_search_hits.is_empty() {
                    ui.separator();
                    ui.label("Search Results:");
                    let hits = self.last_search_hits.clone();
                    for hit in hits {
                        ui.horizontal(|ui| {
                            ui.label(format!("[{}] {}", hit.page_title, hit.snippet));
                            if ui.button("Go").clicked() {
                                let _ = self.session.execute(Command::SectionSelect {
                                    section_id: hit.section_id,
                                });
                                let _ = self.session.execute(Command::PageSelect {
                                    page_id: hit.page_id,
                                });
                            }
                        });
                    }
                }
            });
        });
    }

    fn render_left_section_rail(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("section_rail")
            .resizable(true)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.heading("Sections");
                ui.separator();

                let active_sec = self.session.active_section_id;
                let sections = self.session.notebook.sections.clone();

                for sec in &sections {
                    let is_active = sec.id == active_sec;
                    if ui.selectable_label(is_active, &sec.name).clicked() {
                        let _ = self
                            .session
                            .execute(Command::SectionSelect { section_id: sec.id });
                    }
                }

                ui.separator();

                if ui.button("+ Add Section").clicked() {
                    let sec_count = self.session.notebook.sections.len() + 1;
                    let _ = self.session.execute(Command::SectionCreate {
                        name: format!("Section {}", sec_count),
                        color_hex: "#3B82F6".to_string(),
                    });
                }
            });
    }

    fn render_right_page_hierarchy(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("page_hierarchy")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Pages");
                ui.separator();

                let active_page_id = self.session.active_page_id;
                let pages = match self.session.active_section() {
                    Ok(sec) => sec.pages.clone(),
                    Err(_) => Vec::new(),
                };

                for page in &pages {
                    let indent_space = match page.indent_level {
                        1 => "  ↳ ",
                        2 => "    ↳ ",
                        _ => "",
                    };
                    let label = format!("{}{}", indent_space, page.title);
                    let is_active = page.id == active_page_id;

                    if ui.selectable_label(is_active, label).clicked() {
                        let _ = self
                            .session
                            .execute(Command::PageSelect { page_id: page.id });
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("+ Add Page").clicked() {
                        let page_count = pages.len() + 1;
                        let _ = self.session.execute(Command::PageCreate {
                            title: format!("Page {}", page_count),
                            indent_level: 0,
                        });
                    }
                    if ui.button("+ Subpage").clicked() {
                        let page_count = pages.len() + 1;
                        let _ = self.session.execute(Command::PageCreate {
                            title: format!("Subpage {}", page_count),
                            indent_level: 1,
                        });
                    }
                });
            });
    }

    fn render_center_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(),
                egui::Sense::click_and_drag(),
            );
            let canvas_rect = response.rect;

            // Handle Panning gesture
            if response.dragged_by(egui::PointerButton::Middle)
                || (self.active_tool == CanvasTool::Pan
                    && response.dragged_by(egui::PointerButton::Primary))
            {
                let delta = response.drag_delta();
                self.viewport.pan_by_screen(delta.x, delta.y);
            }

            // Transform helpers
            let to_screen = |pos_mm: PointMm, vp: &ViewportTransform| -> egui::Pos2 {
                let (sx, sy) = vp.mm_to_screen(pos_mm);
                egui::pos2(canvas_rect.min.x + sx, canvas_rect.min.y + sy)
            };

            let to_mm = |pos_screen: egui::Pos2, vp: &ViewportTransform| -> PointMm {
                let local_x = pos_screen.x - canvas_rect.min.x;
                let local_y = pos_screen.y - canvas_rect.min.y;
                vp.screen_to_mm(local_x, local_y)
            };

            let px_per_mm = self.px_per_mm();

            // Active Page Background & Rule lines
            if let Ok(active_page) = self.session.active_page().cloned() {
                match active_page.rule_style {
                    PaperRuleStyle::None => {}
                    PaperRuleStyle::RuledLines { spacing_mm } => {
                        let line_spacing_mm = spacing_mm as f32;
                        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(210, 220, 240));
                        let start_mm = (self.viewport.pan_mm.y / line_spacing_mm).floor()
                            * line_spacing_mm
                            - 50.0;
                        let end_mm = start_mm + (canvas_rect.height() / px_per_mm) + 100.0;

                        let mut y_mm = start_mm;
                        while y_mm <= end_mm {
                            let p1 = to_screen(PointMm::new(-1000.0, y_mm), &self.viewport);
                            let p2 = to_screen(PointMm::new(1000.0, y_mm), &self.viewport);
                            painter.line_segment([p1, p2], stroke);
                            y_mm += line_spacing_mm;
                        }
                    }
                    PaperRuleStyle::Grid { cell_mm } => {
                        let grid_spacing_mm = cell_mm as f32;
                        let stroke = egui::Stroke::new(0.5, egui::Color32::from_rgb(220, 220, 230));

                        let start_x = (self.viewport.pan_mm.x / grid_spacing_mm).floor()
                            * grid_spacing_mm
                            - 50.0;
                        let end_x = start_x + (canvas_rect.width() / px_per_mm) + 100.0;
                        let mut x_mm = start_x;
                        while x_mm <= end_x {
                            let p1 = to_screen(PointMm::new(x_mm, -1000.0), &self.viewport);
                            let p2 = to_screen(PointMm::new(x_mm, 1000.0), &self.viewport);
                            painter.line_segment([p1, p2], stroke);
                            x_mm += grid_spacing_mm;
                        }

                        let start_y = (self.viewport.pan_mm.y / grid_spacing_mm).floor()
                            * grid_spacing_mm
                            - 50.0;
                        let end_y = start_y + (canvas_rect.height() / px_per_mm) + 100.0;
                        let mut y_mm = start_y;
                        while y_mm <= end_y {
                            let p1 = to_screen(PointMm::new(-1000.0, y_mm), &self.viewport);
                            let p2 = to_screen(PointMm::new(1000.0, y_mm), &self.viewport);
                            painter.line_segment([p1, p2], stroke);
                            y_mm += grid_spacing_mm;
                        }
                    }
                    PaperRuleStyle::A4Guide210Mm => {
                        let guide_stroke =
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(239, 68, 68));
                        let p1 = to_screen(PointMm::new(210.0, -1000.0), &self.viewport);
                        let p2 = to_screen(PointMm::new(210.0, 1000.0), &self.viewport);
                        painter.line_segment([p1, p2], guide_stroke);
                    }
                }

                // Render Printouts
                for printout in &active_page.printouts {
                    for page_frame in &printout.pages {
                        let top_left = to_screen(page_frame.bounds_mm.origin, &self.viewport);
                        let size = egui::vec2(
                            page_frame.bounds_mm.size.width * px_per_mm,
                            page_frame.bounds_mm.size.height * px_per_mm,
                        );
                        let rect = egui::Rect::from_min_size(top_left, size);
                        painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(250, 250, 250));
                        painter.rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 180)),
                            egui::StrokeKind::Outside,
                        );
                        painter.text(
                            rect.min + egui::vec2(8.0, 8.0),
                            egui::Align2::LEFT_TOP,
                            format!(
                                "Page {} - {}",
                                page_frame.page_number, page_frame.extracted_text
                            ),
                            egui::FontId::proportional(12.0),
                            egui::Color32::DARK_GRAY,
                        );
                    }
                }

                // Render Embedded Sheets
                for sheet in &active_page.sheets {
                    let top_left = to_screen(sheet.bounds_mm.origin, &self.viewport);
                    let size = egui::vec2(
                        sheet.bounds_mm.size.width * px_per_mm,
                        sheet.bounds_mm.size.height * px_per_mm,
                    );
                    let rect = egui::Rect::from_min_size(top_left, size);
                    painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(1.5, egui::Color32::from_rgb(16, 185, 129)),
                        egui::StrokeKind::Outside,
                    );

                    let rows = sheet.cells.len();
                    if rows > 0 {
                        let row_h = rect.height() / (rows as f32);
                        for (r_idx, row) in sheet.cells.iter().enumerate() {
                            let cols = row.len();
                            if cols > 0 {
                                let col_w = rect.width() / (cols as f32);
                                for (c_idx, cell) in row.iter().enumerate() {
                                    let cell_rect = egui::Rect::from_min_size(
                                        rect.min
                                            + egui::vec2(
                                                (c_idx as f32) * col_w,
                                                (r_idx as f32) * row_h,
                                            ),
                                        egui::vec2(col_w, row_h),
                                    );
                                    painter.rect_stroke(
                                        cell_rect,
                                        0.0,
                                        egui::Stroke::new(
                                            0.5,
                                            egui::Color32::from_rgb(200, 200, 200),
                                        ),
                                        egui::StrokeKind::Outside,
                                    );
                                    painter.text(
                                        cell_rect.min + egui::vec2(4.0, 2.0),
                                        egui::Align2::LEFT_TOP,
                                        cell,
                                        egui::FontId::proportional(11.0),
                                        egui::Color32::BLACK,
                                    );
                                }
                            }
                        }
                    }
                }

                // Render Attachment Badges (45 mm x 14 mm)
                for att in &active_page.attachments {
                    let top_left = to_screen(att.pos_mm, &self.viewport);
                    let size = egui::vec2(45.0 * px_per_mm, 14.0 * px_per_mm);
                    let rect = egui::Rect::from_min_size(top_left, size);
                    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(238, 242, 255));
                    painter.rect_stroke(
                        rect,
                        4.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(99, 102, 241)),
                        egui::StrokeKind::Outside,
                    );
                    painter.text(
                        rect.min + egui::vec2(6.0, 4.0),
                        egui::Align2::LEFT_TOP,
                        format!("📎 {}", att.file_name),
                        egui::FontId::proportional(11.0),
                        egui::Color32::from_rgb(67, 56, 202),
                    );
                }

                // Render Outlines
                for outline in &active_page.outlines {
                    let top_left = to_screen(outline.bounds_mm.origin, &self.viewport);
                    let size = egui::vec2(
                        outline.bounds_mm.size.width * px_per_mm,
                        outline.bounds_mm.size.height * px_per_mm,
                    );
                    let rect = egui::Rect::from_min_size(top_left, size);
                    painter.rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)),
                        egui::StrokeKind::Outside,
                    );

                    let mut text_y = rect.min.y + 4.0;
                    for para in &outline.paragraphs {
                        let tag_str = match &para.tag {
                            Some(NoteTag::Todo { checked: true }) => "[x] ",
                            Some(NoteTag::Todo { checked: false }) => "[ ] ",
                            Some(NoteTag::Important) => "★ ",
                            Some(NoteTag::Question) => "? ",
                            Some(NoteTag::Definition) => "Def: ",
                            Some(NoteTag::Custom { label, .. }) => label.as_str(),
                            None => "",
                        };
                        let line = format!("{}{}", tag_str, para.text);
                        painter.text(
                            egui::pos2(rect.min.x + 6.0, text_y),
                            egui::Align2::LEFT_TOP,
                            line,
                            egui::FontId::proportional(13.0),
                            egui::Color32::BLACK,
                        );
                        text_y += 16.0;
                    }
                }

                // Render Ink Strokes
                for stroke in &active_page.ink_strokes {
                    if stroke.points.len() >= 2 {
                        let color = egui::Color32::from_rgba_unmultiplied(
                            stroke.color_rgba[0],
                            stroke.color_rgba[1],
                            stroke.color_rgba[2],
                            stroke.color_rgba[3],
                        );
                        let width = (stroke.base_thickness_mm * px_per_mm).max(1.0);
                        let egui_stroke = egui::Stroke::new(width, color);

                        for i in 0..(stroke.points.len() - 1) {
                            if let (Some(p1), Some(p2)) =
                                (stroke.points.get(i), stroke.points.get(i + 1))
                            {
                                let s1 = to_screen(p1.pos_mm, &self.viewport);
                                let s2 = to_screen(p2.pos_mm, &self.viewport);
                                painter.line_segment([s1, s2], egui_stroke);
                            }
                        }
                    }
                }
            }

            // Interactive Drawing / Pointer handling
            if response.clicked() {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let point_mm = to_mm(click_pos, &self.viewport);
                    if self.active_tool == CanvasTool::InsertOutline {
                        let _ = self.session.execute(Command::CanvasInsertOutline {
                            x_mm: point_mm.x,
                            y_mm: point_mm.y,
                            width_mm: 100.0,
                            initial_text: "New Outline".to_string(),
                        });
                        self.active_tool = CanvasTool::Select;
                    } else if self.active_tool == CanvasTool::Eraser {
                        let _ = self.session.execute(Command::InkEraseStroke {
                            x_mm: point_mm.x,
                            y_mm: point_mm.y,
                            eraser_radius_mm: self.eraser_radius_mm,
                        });
                    }
                }
            }

            if response.dragged_by(egui::PointerButton::Primary) {
                if let Some(drag_pos) = response.interact_pointer_pos() {
                    let point_mm = to_mm(drag_pos, &self.viewport);
                    if self.active_tool == CanvasTool::Pen
                        || self.active_tool == CanvasTool::Highlighter
                    {
                        self.current_stroke_points
                            .push((point_mm.x, point_mm.y, 1.0));
                    } else if self.active_tool == CanvasTool::Eraser {
                        let _ = self.session.execute(Command::InkEraseStroke {
                            x_mm: point_mm.x,
                            y_mm: point_mm.y,
                            eraser_radius_mm: self.eraser_radius_mm,
                        });
                    }
                }
            }

            if response.drag_stopped_by(egui::PointerButton::Primary)
                && !self.current_stroke_points.is_empty()
            {
                let points = std::mem::take(&mut self.current_stroke_points);
                if self.active_tool == CanvasTool::Pen {
                    let _ = self.session.execute(Command::InkAddStroke {
                        color_rgba: self.pen_color_rgba,
                        thickness_mm: self.pen_thickness_mm,
                        is_highlighter: false,
                        points,
                    });
                } else if self.active_tool == CanvasTool::Highlighter {
                    let _ = self.session.execute(Command::InkAddStroke {
                        color_rgba: self.highlighter_color_rgba,
                        thickness_mm: self.highlighter_thickness_mm,
                        is_highlighter: true,
                        points,
                    });
                }
            }
        });
    }

    pub fn render_offscreen_ppm(&mut self, width: u32, height: u32) -> Vec<u8> {
        let ctx = egui::Context::default();
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width as f32, height as f32),
            )),
            ..Default::default()
        };

        let _ = ctx.run(raw_input, |ctx| {
            self.render_ui(ctx);
        });

        // Software rasterizer for deterministic PPM P6 visual preview
        let mut buffer = vec![245u8; (width * height * 3) as usize];

        let draw_pixel = |buf: &mut [u8], x: u32, y: u32, color: [u8; 3]| {
            if x < width && y < height {
                let idx = ((y * width + x) * 3) as usize;
                if idx + 2 < buf.len() {
                    buf[idx] = color[0];
                    buf[idx + 1] = color[1];
                    buf[idx + 2] = color[2];
                }
            }
        };

        let draw_rect = |buf: &mut [u8], rx: u32, ry: u32, rw: u32, rh: u32, color: [u8; 3]| {
            for y in ry..(ry + rh) {
                for x in rx..(rx + rw) {
                    draw_pixel(buf, x, y, color);
                }
            }
        };

        // Draw Ribbon Bar (Top 40px)
        draw_rect(&mut buffer, 0, 0, width, 40, [220, 225, 230]);
        // Draw Left Section Rail (Width 150px)
        draw_rect(
            &mut buffer,
            0,
            40,
            150.min(width),
            height.saturating_sub(40),
            [235, 238, 242],
        );
        // Draw Right Page Hierarchy (Width 150px)
        let right_x = width.saturating_sub(150);
        if right_x > 150 {
            draw_rect(
                &mut buffer,
                right_x,
                40,
                150,
                height.saturating_sub(40),
                [238, 242, 245],
            );
        }

        let px_per_mm = self.px_per_mm();

        // Draw Canvas Items (Rule line, Outline, Ink Strokes)
        if let Ok(active_page) = self.session.active_page().cloned() {
            if active_page.rule_style == PaperRuleStyle::A4Guide210Mm {
                let guide_x = 150 + (210.0 * px_per_mm) as u32;
                if guide_x < right_x {
                    for y in 40..height {
                        draw_pixel(&mut buffer, guide_x, y, [239, 68, 68]);
                    }
                }
            }

            for outline in &active_page.outlines {
                let ox = 150 + (outline.bounds_mm.origin.x * px_per_mm) as u32;
                let oy = 40 + (outline.bounds_mm.origin.y * px_per_mm) as u32;
                let ow = (outline.bounds_mm.size.width * px_per_mm) as u32;
                let oh = (outline.bounds_mm.size.height * px_per_mm) as u32;
                draw_rect(
                    &mut buffer,
                    ox,
                    oy,
                    ow.min(100),
                    oh.min(40),
                    [200, 220, 255],
                );
            }

            for stroke in &active_page.ink_strokes {
                for pt in &stroke.points {
                    let px = 150 + (pt.pos_mm.x * px_per_mm) as u32;
                    let py = 40 + (pt.pos_mm.y * px_per_mm) as u32;
                    let c = [
                        stroke.color_rgba[0],
                        stroke.color_rgba[1],
                        stroke.color_rgba[2],
                    ];
                    draw_rect(&mut buffer, px, py, 3, 3, c);
                }
            }
        }

        let header = format!("P6\n{} {}\n255\n", width, height);
        let mut ppm_bytes = header.into_bytes();
        ppm_bytes.extend_from_slice(&buffer);
        ppm_bytes
    }

    pub fn save_offscreen_shot(
        &mut self,
        path: &std::path::Path,
        width: u32,
        height: u32,
    ) -> std::io::Result<()> {
        let ppm_data = self.render_offscreen_ppm(width, height);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, ppm_data)
    }
}
