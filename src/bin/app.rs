// Cofre Vault — Desktop UI with semantic graph visualization

use eframe::egui;
use egui::{Color32, FontId, Pos2, Rect, RichText, ScrollArea, Stroke, Ui, Vec2};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use cofre_vault::models::{ContentItem, ContentType, Vault};
use cofre_vault::services::ElevenLabsClient;
use cofre_vault::Error;

// ── Colors ────────────────────────────────────────────────────────────────────
// Warm dark palette — cozy, intimate, not "crypto terminal"
const BG:         Color32 = Color32::from_rgb(24, 22, 32);   // warm deep purple-charcoal
const SIDEBAR:    Color32 = Color32::from_rgb(30, 27, 42);   // slightly lighter sidebar
const CARD:       Color32 = Color32::from_rgb(38, 34, 54);   // warm card surface
const CARD_SEL:   Color32 = Color32::from_rgb(58, 44, 88);   // selected card — soft violet
const BORDER:     Color32 = Color32::from_rgb(60, 54, 82);   // subtle, not harsh
const ACCENT:     Color32 = Color32::from_rgb(167, 139, 250); // pastel lavender
const ACCENT_BTN: Color32 = Color32::from_rgb(124, 92, 210); // softer indigo button
const TEXT:       Color32 = Color32::from_rgb(243, 240, 255); // off-white, easy on eyes
const TEXT_SUB:   Color32 = Color32::from_rgb(180, 170, 210); // muted lavender-grey
const TEXT_DIM:   Color32 = Color32::from_rgb(120, 110, 150); // dimmed secondary text
const GREEN:      Color32 = Color32::from_rgb(110, 220, 150); // soft mint green
const RED:        Color32 = Color32::from_rgb(240, 100, 100); // warm red
const YELLOW:     Color32 = Color32::from_rgb(250, 200, 80);  // warm amber
const BLUE:       Color32 = Color32::from_rgb(120, 185, 255); // soft sky blue
const INPUT_BG:   Color32 = Color32::from_rgb(44, 40, 62);   // input field background — visible but dark
const INPUT_HINT: Color32 = Color32::from_rgb(130, 118, 165); // placeholder text — readable on INPUT_BG

// Graph colors
const NODE_AUDIO: Color32 = Color32::from_rgb(100, 180, 255);
const NODE_LINK:  Color32 = Color32::from_rgb(255, 200, 100);
const NODE_IMAGE: Color32 = Color32::from_rgb(180, 255, 150);

// ── Data structures ───────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct AudioReport {
    pub title: String,
    pub file_path: String,
    pub transcript: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
struct ItemEmbedding {
    item_id: Uuid,
    tfidf: HashMap<String, f32>,
}

#[derive(Clone)]
struct GraphNode {
    item_id: Uuid,
    pos: Pos2,
    vel: Vec2,
}

#[derive(Clone)]
struct GraphEdge {
    from: Uuid,
    to: Uuid,
    weight: f32,
}

// ── Views ─────────────────────────────────────────────────────────────────────
#[derive(PartialEq, Clone, Copy)]
enum View { Spaces, Memories, Graph, Chat, Transcripts }

// ── Chat message ──────────────────────────────────────────────────────────────
#[derive(Clone)]
struct ChatMessage {
    role: String, // "user" | "assistant"
    content: String,
    referenced_nodes: Vec<Uuid>,
    timestamp: chrono::DateTime<Utc>,
}

// ── Batch transcription job ───────────────────────────────────────────────────
struct BatchJob {
    total: usize,
    completed: usize,
    failed: usize,
    current_file: String,
    last_error: Option<String>,
}

// ── App state ─────────────────────────────────────────────────────────────────
pub struct CofreApp {
    view: View,

    vaults: Vec<Vault>,
    selected_vault: Option<usize>,
    new_vault_name: String,
    new_vault_desc: String,

    items: Vec<ContentItem>,
    embeddings: Vec<ItemEmbedding>,
    picked_files: Vec<String>,

    reports: Vec<AudioReport>,
    selected_report: Option<usize>,

    // Batch transcription
    batch_job: Option<BatchJob>,
    job_queue: Vec<(usize, String)>, // (item_idx, file_path)
    job_cell: Option<Arc<Mutex<Option<std::result::Result<String, String>>>>>,

    // Graph visualization
    graph_nodes: Vec<GraphNode>,
    graph_edges: Vec<GraphEdge>,
    graph_needs_rebuild: bool,
    selected_node: Option<Uuid>,
    similarity_threshold: f32,
    
    // Chat
    chat_messages: Vec<ChatMessage>,
    chat_input: String,
    chat_loading: bool,
    chat_referenced_nodes: Vec<Uuid>,
    chat_result_cell: Option<Arc<Mutex<Option<Result<cofre_vault::models::ChatResponse, String>>>>>,

    status: Option<(String, bool)>,
    user_id: Uuid,
    rt: tokio::runtime::Handle,
}

impl CofreApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();
        std::mem::forget(rt);
        Self {
            view: View::Spaces,
            vaults: Vec::new(), selected_vault: None,
            new_vault_name: String::new(), new_vault_desc: String::new(),
            items: Vec::new(), embeddings: Vec::new(),
            picked_files: Vec::new(),
            reports: Vec::new(), selected_report: None,
            batch_job: None, job_queue: Vec::new(), job_cell: None,
            graph_nodes: Vec::new(), graph_edges: Vec::new(),
            graph_needs_rebuild: true, selected_node: None,
            similarity_threshold: 0.5,
            chat_messages: Vec::new(), chat_input: String::new(),
            chat_loading: false, chat_referenced_nodes: Vec::new(),
            chat_result_cell: None,
            status: None, user_id: Uuid::new_v4(), rt: handle,
        }
    }

    fn ok(&mut self, msg: impl Into<String>)  { self.status = Some((msg.into(), false)); }
    fn err(&mut self, msg: impl Into<String>) { self.status = Some((msg.into(), true)); }

    fn vault_id(&self) -> Option<Uuid> {
        self.selected_vault.and_then(|i| self.vaults.get(i)).map(|v| v.id)
    }

    // TF-IDF based similarity
    fn compute_tfidf(&self, text: &str) -> HashMap<String, f32> {
        let words: Vec<String> = text.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect();
        
        let mut tf: HashMap<String, f32> = HashMap::new();
        for word in &words {
            *tf.entry(word.clone()).or_insert(0.0) += 1.0;
        }
        
        // Normalize by doc length
        let total = words.len() as f32;
        for val in tf.values_mut() {
            *val /= total;
        }
        
        tf
    }

    fn cosine_similarity_tfidf(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
        let mut dot = 0.0;
        let mut mag_a = 0.0;
        let mut mag_b = 0.0;
        
        for (word, val_a) in a {
            mag_a += val_a * val_a;
            if let Some(val_b) = b.get(word) {
                dot += val_a * val_b;
            }
        }
        for val_b in b.values() {
            mag_b += val_b * val_b;
        }
        
        if mag_a == 0.0 || mag_b == 0.0 { 0.0 } else { dot / (mag_a.sqrt() * mag_b.sqrt()) }
    }
    
    // Cosine similarity for real embedding vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() { return 0.0; }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 { 0.0 } else { dot / (mag_a * mag_b) }
    }
}

// ── Style ─────────────────────────────────────────────────────────────────────
fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let v = &mut style.visuals;
    v.dark_mode = true;
    v.panel_fill = BG;
    v.window_fill = CARD;
    v.override_text_color = Some(TEXT);
    v.widgets.noninteractive.bg_fill = CARD;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SUB);
    v.widgets.noninteractive.rounding = 10.0.into();
    v.widgets.inactive.bg_fill = INPUT_BG;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SUB);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.rounding = 10.0.into();
    v.widgets.hovered.bg_fill = Color32::from_rgb(60, 50, 90);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.hovered.rounding = 10.0.into();
    v.widgets.active.bg_fill = ACCENT_BTN;
    v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.active.rounding = 10.0.into();
    v.selection.bg_fill = ACCENT_BTN;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.window_rounding = 14.0.into();
    v.window_stroke = Stroke::new(1.0, BORDER);
    // Softer separators
    v.widgets.noninteractive.bg_stroke = Stroke::new(0.5, BORDER);
    ctx.set_style(style);
}

// ── Main loop ─────────────────────────────────────────────────────────────────
impl eframe::App for CofreApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_style(ctx);
        self.poll_batch_job(ctx);
        self.poll_chat_result();
        if self.graph_needs_rebuild && self.view == View::Graph {
            self.rebuild_graph();
        }

        egui::SidePanel::left("nav")
            .exact_width(190.0).resizable(false)
            .frame(egui::Frame::none().fill(SIDEBAR).inner_margin(14.0))
            .show(ctx, |ui| self.draw_sidebar(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG).inner_margin(egui::Margin {
                left: 24.0, right: 24.0, top: 20.0, bottom: 20.0
            }))
            .show(ctx, |ui| {
                match self.view {
                    View::Spaces      => self.show_vaults(ui),
                    View::Memories    => self.show_content(ui, ctx),
                    View::Graph       => self.show_graph(ui),
                    View::Chat        => self.show_chat(ui, ctx),
                    View::Transcripts => self.show_reports(ui),
                }
            });
    }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────
impl CofreApp {
    fn draw_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.label(RichText::new("🔐  Cofre").size(20.0).color(ACCENT).strong());
        ui.add_space(2.0);
        ui.label(RichText::new("just for us ✨").size(11.0).color(TEXT_DIM));
        ui.add_space(18.0);

        if let Some(v) = self.selected_vault.and_then(|i| self.vaults.get(i)) {
            egui::Frame::none().fill(CARD_SEL).rounding(10.0).inner_margin(10.0).show(ui, |ui| {
                ui.label(RichText::new("active space").size(10.0).color(TEXT_DIM));
                ui.label(RichText::new(&v.name).size(13.0).color(ACCENT).strong());
            });
            ui.add_space(12.0);
        }

        ui.add_space(4.0);

        if nav(ui, "🏠  Our Spaces",      self.view == View::Spaces)      { self.view = View::Spaces; }
        if nav(ui, "💜  Memories",         self.view == View::Memories)   { self.view = View::Memories; }
        if nav(ui, "🕸  Graph",            self.view == View::Graph)      { self.view = View::Graph; }
        if nav(ui, "💬  Chat",             self.view == View::Chat)       { self.view = View::Chat; }
        if nav(ui, "📝  Transcripts",      self.view == View::Transcripts){ self.view = View::Transcripts; }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(8.0);
            if let Some((msg, is_err)) = &self.status {
                let color = if *is_err { RED } else { GREEN };
                egui::Frame::none().fill(CARD).rounding(8.0).inner_margin(8.0).show(ui, |ui| {
                    ui.label(RichText::new(msg).size(11.0).color(color));
                });
            }
        });
    }
}

fn nav(ui: &mut Ui, label: &str, active: bool) -> bool {
    let (fg, bg) = if active { (Color32::WHITE, ACCENT_BTN) } else { (TEXT_SUB, Color32::TRANSPARENT) };
    let btn = egui::Button::new(RichText::new(label).color(fg).size(13.0))
        .fill(bg).rounding(10.0).min_size(Vec2::new(166.0, 36.0));
    let r = ui.add(btn);
    ui.add_space(3.0);
    r.clicked()
}

fn heading(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).size(24.0).color(TEXT).strong());
    ui.add_space(16.0);
}

fn card(ui: &mut Ui, selected: bool, add_contents: impl FnOnce(&mut Ui)) {
    let fill = if selected { CARD_SEL } else { CARD };
    let stroke = if selected {
        Stroke::new(1.5, ACCENT)
    } else {
        Stroke::new(0.0, Color32::TRANSPARENT) // no border — use shadow/fill contrast instead
    };
    egui::Frame::none().fill(fill).rounding(14.0).inner_margin(16.0)
        .stroke(stroke)
        .show(ui, add_contents);
    ui.add_space(8.0);
}

fn accent_btn(ui: &mut Ui, label: &str) -> bool {
    ui.add(egui::Button::new(RichText::new(label).color(Color32::WHITE).size(13.0))
        .fill(ACCENT_BTN).rounding(10.0).min_size(Vec2::new(0.0, 32.0))).clicked()
}

fn ghost_btn(ui: &mut Ui, label: &str, color: Color32) -> bool {
    ui.add(egui::Button::new(RichText::new(label).color(color).size(13.0))
        .fill(Color32::TRANSPARENT).rounding(8.0)).clicked()
}

fn text_input<'a>(text: &'a mut String, hint: &str, width: f32) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(text)
        .desired_width(width)
        .hint_text(RichText::new(hint).color(INPUT_HINT).size(13.0))
        .text_color(TEXT)
        .font(FontId::proportional(13.0))
        .margin(egui::Vec2::new(12.0, 8.0)) // chunky padding
}

fn badge(ui: &mut Ui, label: &str, color: Color32) {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30))
        .rounding(4.0).inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 2.0, bottom: 2.0 })
        .show(ui, |ui| { ui.label(RichText::new(label).size(11.0).color(color)); });
}

// ── Vaults view ───────────────────────────────────────────────────────────────
impl CofreApp {
    fn show_vaults(&mut self, ui: &mut Ui) {
        heading(ui, "🏠  Our Spaces");

        card(ui, false, |ui| {
            ui.label(RichText::new("Create a new space").color(ACCENT).size(13.0).strong());
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add(text_input(&mut self.new_vault_name, "Name (required)", 220.0));
                ui.add(text_input(&mut self.new_vault_desc, "Description (optional)", 280.0));
                if accent_btn(ui, "＋ Create") {
                    let name = self.new_vault_name.trim().to_string();
                    if name.is_empty() || name.len() > 100 {
                        self.err("Name must be 1–100 characters");
                    } else {
                        let desc = self.new_vault_desc.trim().to_string();
                        self.vaults.push(Vault {
                            id: Uuid::new_v4(), name,
                            description: if desc.is_empty() { None } else { Some(desc) },
                            created_by: self.user_id, created_at: Utc::now(),
                        });
                        self.new_vault_name.clear(); self.new_vault_desc.clear();
                        self.ok("✨ Space created");
                    }
                }
            });
        });

        if self.vaults.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🏡").size(48.0));
                ui.add_space(12.0);
                ui.label(RichText::new("No spaces yet").size(16.0).color(TEXT_SUB));
                ui.add_space(6.0);
                ui.label(RichText::new("Create one above to start saving memories together 💜")
                    .size(13.0).color(TEXT_DIM));
            });
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            let mut sel: Option<usize> = None;
            let mut del: Option<usize> = None;
            for (i, v) in self.vaults.iter().enumerate() {
                let active = self.selected_vault == Some(i);
                let name = v.name.clone();
                let desc = v.description.clone();
                let n_items = self.items.iter().filter(|it| it.vault_id == v.id).count();
                card(ui, active, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&name).size(16.0).color(TEXT).strong());
                            if let Some(d) = &desc {
                                ui.label(RichText::new(d).size(12.0).color(TEXT_SUB));
                            }
                            ui.add_space(4.0);
                            badge(ui, &format!("💜 {} memories", n_items), BLUE);
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ghost_btn(ui, "🗑 Delete", RED) { del = Some(i); }
                            ui.add_space(6.0);
                            if active {
                                ui.label(RichText::new("✓ Active").color(GREEN).size(12.0));
                            } else if accent_btn(ui, "Open") {
                                sel = Some(i);
                            }
                        });
                    });
                });
            }
            if let Some(i) = sel {
                let n = self.vaults[i].name.clone();
                self.selected_vault = Some(i);
                self.ok(format!("'{}' opened", n));
            }
            if let Some(i) = del {
                if self.selected_vault == Some(i) { self.selected_vault = None; }
                self.vaults.remove(i);
                self.ok("Space deleted");
            }
        });
    }
}

// ── Content view (multi-file upload) ──────────────────────────────────────────
impl CofreApp {
    fn show_content(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        heading(ui, "💜  Memories");

        let vid = match self.vault_id() {
            Some(id) => id,
            None => {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🏠").size(48.0));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Open a space first").size(16.0).color(TEXT_SUB));
                    ui.add_space(6.0);
                    ui.label(RichText::new("Go to «Our Spaces» and select one to get started 💜")
                        .size(13.0).color(TEXT_DIM));
                });
                return;
            }
        };
        let vname = self.vaults[self.selected_vault.unwrap()].name.clone();
        ui.label(RichText::new(format!("📂  {}", vname)).color(ACCENT).size(13.0));
        ui.add_space(10.0);

        // ── Multi-file picker ─────────────────────────────────────────────────
        card(ui, false, |ui| {
            ui.label(RichText::new("Upload voice notes or audio").color(ACCENT).size(13.0).strong());
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("🎙 Pick files").color(TEXT).size(13.0))
                    .fill(CARD_SEL).rounding(10.0).min_size(Vec2::new(140.0, 34.0))).clicked()
                {
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("Audio", &["mp3","wav","webm","ogg","aac","flac","m4a"])
                        .pick_files()
                    {
                        self.picked_files = paths.iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                    }
                }

                if !self.picked_files.is_empty() {
                    ui.label(RichText::new(format!("{} files ready", self.picked_files.len()))
                        .color(GREEN).size(13.0));
                    if accent_btn(ui, "＋ Add all") {
                        for path in &self.picked_files {
                            let title = std::path::Path::new(path)
                                .file_stem().and_then(|s| s.to_str())
                                .map(|s| s.to_string());
                            self.items.push(ContentItem {
                                id: Uuid::new_v4(), vault_id: vid,
                                created_by: self.user_id,
                                content_type: ContentType::Audio,
                                title, url: path.clone(),
                                transcript: None, metadata: None,
                                created_at: Utc::now(),
                            });
                        }
                        let n = self.picked_files.len();
                        self.picked_files.clear();
                        self.ok(format!("{} memories added ✨", n));
                        self.graph_needs_rebuild = true;
                    }
                    if ghost_btn(ui, "Clear", RED) {
                        self.picked_files.clear();
                    }
                }
            });

            if !self.picked_files.is_empty() {
                ui.add_space(6.0);
                ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                    for p in &self.picked_files {
                        let name = std::path::Path::new(p).file_name()
                            .and_then(|n| n.to_str()).unwrap_or(p);
                        ui.label(RichText::new(format!("• {}", name)).size(11.0).color(TEXT_SUB));
                    }
                });
            }
        });

        // ── Batch transcription ───────────────────────────────────────────────
        let indices: Vec<usize> = self.items.iter().enumerate()
            .filter(|(_, it)| it.vault_id == vid)
            .map(|(i, _)| i)
            .collect();

        let untranscribed: Vec<usize> = indices.iter().copied()
            .filter(|&i| self.items[i].content_type == ContentType::Audio
                && self.items[i].transcript.is_none())
            .collect();

        if !untranscribed.is_empty() && self.batch_job.is_none() {
            card(ui, false, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{} audio files without transcripts", untranscribed.len()))
                        .color(YELLOW).size(13.0));
                    if accent_btn(ui, "🎙 Transcribe all") {
                        self.start_batch_transcription(untranscribed);
                    }
                });
            });
        }

        if let Some(job) = &self.batch_job {
            card(ui, false, |ui| {
                ui.label(RichText::new("Transcribing...").color(ACCENT).size(13.0).strong());
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new(format!("{} / {} completed", job.completed, job.total))
                        .color(TEXT).size(13.0));
                });
                if !job.current_file.is_empty() {
                    ui.label(RichText::new(format!("Processing: {}", job.current_file))
                        .size(11.0).color(TEXT_DIM));
                }
                if job.failed > 0 {
                    ui.label(RichText::new(format!("{} failed", job.failed)).color(RED).size(11.0));
                    if let Some(err) = &job.last_error {
                        ui.label(RichText::new(format!("Last error: {}", err)).color(RED).size(10.0));
                    }
                }
                let progress = job.completed as f32 / job.total as f32;
                ui.add(egui::ProgressBar::new(progress).desired_width(ui.available_width()));
            });
        }

        // ── Item list ─────────────────────────────────────────────────────────
        if indices.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🎙").size(48.0));
                ui.add_space(12.0);
                ui.label(RichText::new("It's quiet in here...").size(16.0).color(TEXT_SUB));
                ui.add_space(6.0);
                ui.label(RichText::new("Drop a voice note or audio file to save this moment 💜")
                    .size(13.0).color(TEXT_DIM));
            });
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            let mut del: Option<usize> = None;
            let mut transcribe_single: Option<usize> = None;

            for &idx in &indices {
                let item = &self.items[idx];
                let (icon, icolor) = match item.content_type {
                    ContentType::Audio => ("🎵", NODE_AUDIO),
                    ContentType::Image => ("🖼", NODE_IMAGE),
                    ContentType::Link  => ("🔗", NODE_LINK),
                };
                let title = item.title.clone().unwrap_or_else(|| item.url.clone());
                let has_transcript = item.transcript.is_some();

                card(ui, false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(22.0).color(icolor));
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&title).size(14.0).color(TEXT).strong());
                            ui.label(RichText::new(&item.url).size(11.0).color(TEXT_DIM));
                            if has_transcript {
                                ui.label(RichText::new("✓ Transcript available").size(11.0).color(GREEN));
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ghost_btn(ui, "🗑", RED) { del = Some(idx); }
                            if item.content_type == ContentType::Audio && !has_transcript
                                && self.batch_job.is_none()
                            {
                                if accent_btn(ui, "🎙 Transcribe") {
                                    transcribe_single = Some(idx);
                                }
                            }
                        });
                    });
                });
            }

            if let Some(idx) = del {
                let item_id = self.items[idx].id;
                self.embeddings.retain(|e| e.item_id != item_id);
                self.items.remove(idx);
                self.ok("Memory deleted");
                self.graph_needs_rebuild = true;
            }
            if let Some(idx) = transcribe_single {
                self.start_batch_transcription(vec![idx]);
            }
        });
    }

    fn start_batch_transcription(&mut self, indices: Vec<usize>) {
        let total = indices.len();
        self.job_queue = indices.into_iter()
            .map(|i| (i, self.items[i].url.clone()))
            .collect();
        self.batch_job = Some(BatchJob {
            total, completed: 0, failed: 0, current_file: String::new(), last_error: None,
        });
        self.ok(format!("Transcribing {} files... 🎙", total));
    }

    fn poll_batch_job(&mut self, ctx: &egui::Context) {
        if self.batch_job.is_none() { return; }

        // Check if current job finished
        let mut job_done = false;
        let mut transcript_result: Option<(usize, std::result::Result<String, String>)> = None;

        if let Some(cell) = &self.job_cell {
            if let Some(result) = cell.lock().unwrap().take() {
                if let Some((idx, _)) = self.job_queue.first() {
                    transcript_result = Some((*idx, result));
                }
                job_done = true;
            }
        }

        if job_done {
            self.job_cell = None;
            if let Some((idx, result)) = transcript_result {
                match result {
                    Ok(text) => {
                        self.items[idx].transcript = Some(text.clone());
                        // Create report
                        let title = self.items[idx].title.clone()
                            .unwrap_or_else(|| self.items[idx].url.clone());
                        let now = Utc::now();
                        self.reports.push(AudioReport {
                            title: title.clone(),
                            file_path: self.items[idx].url.clone(),
                            transcript: text.clone(),
                            created_at: now,
                        });
                        // Persist report as local .md file inside reports/ folder
                        let audio_stem = std::path::Path::new(&self.items[idx].url)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("transcript");
                        let reports_dir = std::path::Path::new("reports");
                        let _ = std::fs::create_dir_all(reports_dir);
                        let md_path = reports_dir.join(format!("{}.md", audio_stem));
                        let md_content = format!(
                            "# {}\n\n**Archivo:** {}\n**Fecha:** {}\n\n---\n\n## Transcripción\n\n{}\n",
                            title,
                            self.items[idx].url,
                            now.format("%Y-%m-%d %H:%M:%S UTC"),
                            text
                        );
                        if let Err(e) = std::fs::write(&md_path, &md_content) {
                            eprintln!("Error guardando reporte local {}: {}", md_path.display(), e);
                        }
                        // Compute TF-IDF for in-memory graph similarity (desktop-only).
                        // Note: vector embeddings (OpenAI text-embedding-3-small) are generated
                        // via the API server's `EmbeddingService::generate_embedding` +
                        // `db::upsert_embedding` flow, or via `backfill_embeddings` in
                        // `src/services/embedding.rs`. The desktop app has no PgPool, so
                        // pgvector embeddings are not stored here.
                        let tfidf = self.compute_tfidf(&text);
                        self.embeddings.push(ItemEmbedding {
                            item_id: self.items[idx].id,
                            tfidf,
                        });
                        if let Some(job) = &mut self.batch_job {
                            job.completed += 1;
                        }
                    }
                    Err(err_msg) => {
                        eprintln!("Transcription failed: {}", err_msg);
                        if let Some(job) = &mut self.batch_job {
                            job.failed += 1;
                            job.last_error = Some(err_msg);
                            job.completed += 1;
                        }
                    }
                }
                self.job_queue.remove(0);
                self.graph_needs_rebuild = true;
            }
        }

        // Start next job if queue not empty
        if self.job_cell.is_none() && !self.job_queue.is_empty() {
            let (_idx, path) = self.job_queue[0].clone();
            let fname = std::path::Path::new(&path).file_name()
                .and_then(|n| n.to_str()).unwrap_or(&path).to_string();
            self.batch_job.as_mut().unwrap().current_file = fname;

            let api_key = std::env::var("ELEVENLABS_API_KEY").unwrap_or_default();
            let ctx2 = ctx.clone();
            let cell: Arc<Mutex<Option<std::result::Result<String, String>>>> =
                Arc::new(Mutex::new(None));
            let cell2 = cell.clone();

            self.rt.spawn(async move {
                let res = transcribe_file(&path, &api_key).await;
                *cell2.lock().unwrap() = Some(res);
                ctx2.request_repaint();
            });

            self.job_cell = Some(cell);
        }

        // Job finished
        if self.job_queue.is_empty() && self.job_cell.is_none() {
            let job = self.batch_job.take().unwrap();
            if job.failed > 0 {
                let msg = if let Some(err) = &job.last_error {
                    format!("⚠ Done: {} transcribed, {} failed. Last error: {}", 
                        job.completed - job.failed, job.failed, err)
                } else {
                    format!("⚠ Done: {} transcribed, {} failed", 
                        job.completed - job.failed, job.failed)
                };
                self.err(msg);
            } else {
                self.ok(format!("✨ Done: {} transcribed", job.completed));
            }
        }
    }
}

// ── Graph view (force-directed layout) ────────────────────────────────────────
impl CofreApp {
    fn show_graph(&mut self, ui: &mut Ui) {
        heading(ui, "🕸  Semantic Graph");

        let vid = match self.vault_id() {
            Some(id) => id,
            None => {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🏠").size(48.0));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Open a space first").size(16.0).color(TEXT_SUB));
                });
                return;
            }
        };

        let items: Vec<_> = self.items.iter().filter(|it| it.vault_id == vid).collect();
        let n_edges = self.graph_edges.len();

        ui.horizontal(|ui| {
            badge(ui, &format!("{} nodes", items.len()), BLUE);
            badge(ui, &format!("{} edges", n_edges), ACCENT);
            if accent_btn(ui, "🔄 Rebuild") {
                self.graph_needs_rebuild = true;
            }
            ui.add_space(10.0);
            ui.label(RichText::new("Threshold:").size(12.0).color(TEXT_SUB));
            ui.add(egui::Slider::new(&mut self.similarity_threshold, 0.0..=1.0)
                .step_by(0.05)
                .show_value(true));
            if ui.button("Apply").clicked() {
                self.graph_needs_rebuild = true;
            }
        });
        ui.add_space(10.0);

        if items.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🕸").size(48.0));
                ui.add_space(12.0);
                ui.label(RichText::new("The graph is empty").size(16.0).color(TEXT_SUB));
                ui.add_space(6.0);
                ui.label(RichText::new("Add audio and transcribe it to see how memories connect 💜")
                    .size(13.0).color(TEXT_DIM));
            });
            return;
        }

        if self.embeddings.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🎙").size(48.0));
                ui.add_space(12.0);
                ui.label(RichText::new("Transcribe audio files to build the graph").size(16.0).color(TEXT_SUB));
            });
            return;
        }

        // Graph canvas
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), 500.0),
            egui::Sense::click_and_drag()
        );
        let rect = response.rect;
        painter.rect_filled(rect, 0.0, Color32::from_rgb(10, 10, 15));

        // Physics simulation (simple force-directed)
        self.simulate_forces(rect);

        // Draw edges
        for edge in &self.graph_edges {
            if let (Some(n1), Some(n2)) = (
                self.graph_nodes.iter().find(|n| n.item_id == edge.from),
                self.graph_nodes.iter().find(|n| n.item_id == edge.to),
            ) {
                let alpha = (edge.weight * 255.0).min(255.0) as u8;
                let base_color = Color32::from_rgb(140, 100, 255);
                let color = Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), alpha);
                painter.line_segment([n1.pos, n2.pos], Stroke::new(edge.weight * 3.0, color));
            }
        }

        // Draw nodes
        for node in &self.graph_nodes {
            if let Some(item) = self.items.iter().find(|it| it.id == node.item_id) {
                let selected = self.selected_node == Some(node.item_id);
                let is_referenced = self.chat_referenced_nodes.contains(&node.item_id);
                
                let color = match item.content_type {
                    ContentType::Audio => NODE_AUDIO,
                    ContentType::Link  => NODE_LINK,
                    ContentType::Image => NODE_IMAGE,
                };
                let radius = if selected { 12.0 } else { 8.0 };
                
                // Glow effect for referenced nodes
                if is_referenced {
                    painter.circle_filled(node.pos, radius + 6.0, Color32::from_rgba_unmultiplied(167, 139, 250, 40));
                    painter.circle_filled(node.pos, radius + 4.0, Color32::from_rgba_unmultiplied(167, 139, 250, 80));
                }
                
                painter.circle_filled(node.pos, radius, color);
                if selected {
                    painter.circle_stroke(node.pos, radius + 2.0, Stroke::new(2.0, Color32::WHITE));
                }
                if is_referenced && !selected {
                    painter.circle_stroke(node.pos, radius + 2.0, Stroke::new(2.0, ACCENT));
                }

                // Label
                let title = item.title.as_deref().unwrap_or("untitled");
                let short = if title.len() > 20 {
                    format!("{}…", &title[..20])
                } else {
                    title.to_string()
                };
                painter.text(
                    node.pos + Vec2::new(0.0, 16.0),
                    egui::Align2::CENTER_TOP,
                    short,
                    FontId::proportional(10.0),
                    TEXT_SUB,
                );

                // Click detection
                let node_rect = Rect::from_center_size(node.pos, Vec2::splat(radius * 2.0));
                if response.clicked() {
                    if let Some(click_pos) = response.interact_pointer_pos() {
                        if node_rect.contains(click_pos) {
                            self.selected_node = Some(node.item_id);
                        }
                    }
                }
            }
        }

        // Selected node info
        if let Some(nid) = self.selected_node {
            if let Some(item) = self.items.iter().find(|it| it.id == nid) {
                ui.add_space(10.0);
                card(ui, true, |ui| {
                    ui.label(RichText::new("Selected node").color(ACCENT).size(12.0).strong());
                    ui.add_space(4.0);
                    let title = item.title.as_deref().unwrap_or(&item.url);
                    ui.label(RichText::new(title).size(14.0).color(TEXT).strong());
                    if let Some(t) = &item.transcript {
                        ui.add_space(4.0);
                        let preview = if t.len() > 150 {
                            format!("{}…", &t[..150])
                        } else {
                            t.clone()
                        };
                        ui.label(RichText::new(preview).size(11.0).color(TEXT_SUB));
                    }
                    ui.add_space(6.0);
                    if ghost_btn(ui, "Deselect", TEXT_DIM) {
                        self.selected_node = None;
                    }
                });
            }
        }
    }

    fn rebuild_graph(&mut self) {
        self.graph_nodes.clear();
        self.graph_edges.clear();

        let vid = match self.vault_id() {
            Some(id) => id,
            None => return,
        };

        // Create nodes for items with embeddings
        let items_with_emb: Vec<_> = self.items.iter()
            .filter(|it| it.vault_id == vid)
            .filter(|it| self.embeddings.iter().any(|e| e.item_id == it.id))
            .collect();

        if items_with_emb.is_empty() {
            self.graph_needs_rebuild = false;
            return;
        }

        // Initialize node positions (circular layout)
        let n = items_with_emb.len();
        let center = Pos2::new(400.0, 250.0);
        let radius = 150.0;
        for (i, item) in items_with_emb.iter().enumerate() {
            let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
            let pos = center + Vec2::new(angle.cos() * radius, angle.sin() * radius);
            self.graph_nodes.push(GraphNode {
                item_id: item.id,
                pos,
                vel: Vec2::ZERO,
            });
        }

        // Create edges based on embedding similarity
        for i in 0..items_with_emb.len() {
            for j in (i + 1)..items_with_emb.len() {
                let id1 = items_with_emb[i].id;
                let id2 = items_with_emb[j].id;
                if let (Some(e1), Some(e2)) = (
                    self.embeddings.iter().find(|e| e.item_id == id1),
                    self.embeddings.iter().find(|e| e.item_id == id2),
                ) {
                    let sim = Self::cosine_similarity_tfidf(&e1.tfidf, &e2.tfidf);
                    // Only create edge if similarity > threshold
                    if sim > self.similarity_threshold {
                        self.graph_edges.push(GraphEdge {
                            from: id1, to: id2, weight: sim,
                        });
                    }
                }
            }
        }

        self.graph_needs_rebuild = false;
    }

    fn simulate_forces(&mut self, rect: Rect) {
        let dt = 0.016; // ~60fps
        let center = rect.center();
        let repulsion = 2000.0;
        let attraction = 0.01;
        let damping = 0.85;

        // Repulsion between nodes
        for i in 0..self.graph_nodes.len() {
            for j in (i + 1)..self.graph_nodes.len() {
                let delta = self.graph_nodes[j].pos - self.graph_nodes[i].pos;
                let dist = delta.length().max(1.0);
                let force = repulsion / (dist * dist);
                let dir = delta.normalized();
                self.graph_nodes[i].vel -= dir * force * dt;
                self.graph_nodes[j].vel += dir * force * dt;
            }
        }

        // Attraction along edges
        for edge in &self.graph_edges {
            if let (Some(i), Some(j)) = (
                self.graph_nodes.iter().position(|n| n.item_id == edge.from),
                self.graph_nodes.iter().position(|n| n.item_id == edge.to),
            ) {
                let delta = self.graph_nodes[j].pos - self.graph_nodes[i].pos;
                let dist = delta.length();
                let force = attraction * dist * edge.weight;
                let dir = delta.normalized();
                self.graph_nodes[i].vel += dir * force * dt;
                self.graph_nodes[j].vel -= dir * force * dt;
            }
        }

        // Center gravity
        for node in &mut self.graph_nodes {
            let to_center = center - node.pos;
            node.vel += to_center * 0.001;
        }

        // Update positions
        for node in &mut self.graph_nodes {
            node.vel *= damping;
            node.pos += node.vel * dt;
            // Keep in bounds
            node.pos.x = node.pos.x.clamp(rect.min.x + 20.0, rect.max.x - 20.0);
            node.pos.y = node.pos.y.clamp(rect.min.y + 20.0, rect.max.y - 20.0);
        }
    }
}

// ── Reports view ──────────────────────────────────────────────────────────────
impl CofreApp {
    fn show_reports(&mut self, ui: &mut Ui) {
        heading(ui, "📝  Transcripts");

        if self.reports.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("📝").size(48.0));
                ui.add_space(12.0);
                ui.label(RichText::new("No transcripts yet").size(16.0).color(TEXT_SUB));
                ui.add_space(6.0);
                ui.label(RichText::new("Add audio files and transcribe them to read them here 💜")
                    .size(13.0).color(TEXT_DIM));
            });
            return;
        }

        ui.columns(2, |cols| {
            // Left: report list
            ScrollArea::vertical().id_source("rep_list").show(&mut cols[0], |ui| {
                let mut sel: Option<usize> = None;
                let mut del: Option<usize> = None;
                for (i, r) in self.reports.iter().enumerate() {
                    let active = self.selected_report == Some(i);
                    let title = r.title.clone();
                    let date = r.created_at.format("%d %b %Y  %H:%M").to_string();
                    let words = r.transcript.split_whitespace().count();
                    card(ui, active, |ui| {
                        ui.label(RichText::new(&title).size(13.0).color(TEXT).strong());
                        ui.label(RichText::new(&date).size(11.0).color(TEXT_DIM));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            badge(ui, &format!("{} words", words), BLUE);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ghost_btn(ui, "🗑", RED) { del = Some(i); }
                                if !active && accent_btn(ui, "Read") { sel = Some(i); }
                            });
                        });
                    });
                }
                if let Some(i) = sel { self.selected_report = Some(i); }
                if let Some(i) = del {
                    if self.selected_report == Some(i) { self.selected_report = None; }
                    self.reports.remove(i);
                    self.ok("Transcript deleted");
                }
            });

            // Right: report detail
            if let Some(idx) = self.selected_report {
                if let Some(r) = self.reports.get(idx) {
                    let title = r.title.clone();
                    let path  = r.file_path.clone();
                    let date  = r.created_at.format("%d %b %Y  %H:%M:%S").to_string();
                    let text  = r.transcript.clone();

                    cols[1].label(RichText::new(&title).size(18.0).color(TEXT).strong());
                    cols[1].add_space(4.0);
                    cols[1].label(RichText::new(&date).size(11.0).color(TEXT_DIM));
                    cols[1].label(RichText::new(&path).size(11.0).color(TEXT_DIM));
                    cols[1].add_space(10.0);

                    egui::Frame::none().fill(CARD).rounding(10.0).inner_margin(14.0)
                        .stroke(Stroke::new(1.0, BORDER))
                        .show(&mut cols[1], |ui| {
                            ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                                ui.label(RichText::new(&text).size(14.0).color(TEXT).line_height(Some(22.0)));
                            });
                        });

                    cols[1].add_space(10.0);
                    cols[1].horizontal(|ui| {
                        if accent_btn(ui, "📋 Copy transcript") {
                            ui.output_mut(|o| o.copied_text = text.clone());
                            self.status = Some(("Copied ✓".to_string(), false));
                        }
                    });
                }
            } else {
                cols[1].add_space(40.0);
                cols[1].label(RichText::new("← Select a transcript to read it").color(TEXT_DIM));
            }
        });
    }
}

// ── Transcription helper ──────────────────────────────────────────────────────
async fn transcribe_file(path: &str, api_key: &str) -> std::result::Result<String, String> {
    use std::path::Path;
    if api_key.is_empty() {
        return Err("ELEVENLABS_API_KEY not set in .env".to_string());
    }
    let data = std::fs::read(path).map_err(|e| format!("Cannot read file: {}", e))?;
    let ext = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("webm");
    let mime = match ext {
        "mp3"  => "audio/mp3",  "wav"  => "audio/wav",
        "webm" => "audio/webm", "ogg"  => "audio/ogg",
        "aac"  => "audio/aac",  "flac" => "audio/flac",
        "m4a"  => "audio/mp4",  _      => "audio/webm",
    };
    ElevenLabsClient::new(api_key.to_string())
        .transcribe(data, mime).await
        .map_err(|e: Error| e.to_string())
}

// ── Chat view ─────────────────────────────────────────────────────────────────
impl CofreApp {
    fn show_chat(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        heading(ui, "💬  RAG Chat");

        let vid = match self.vault_id() {
            Some(id) => id,
            None => {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🏠").size(48.0));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Open a space first").size(16.0).color(TEXT_SUB));
                });
                return;
            }
        };

        // Chat history
        let available_height = ui.available_height() - 100.0;
        egui::Frame::none()
            .fill(CARD)
            .rounding(12.0)
            .inner_margin(16.0)
            .stroke(Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_source("chat_history")
                    .max_height(available_height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.chat_messages.is_empty() {
                            ui.add_space(40.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("💬").size(48.0));
                                ui.add_space(12.0);
                                ui.label(RichText::new("Start a conversation").size(16.0).color(TEXT_SUB));
                                ui.add_space(6.0);
                                ui.label(RichText::new("Ask questions about your vault content")
                                    .size(13.0).color(TEXT_DIM));
                            });
                        } else {
                            for msg in &self.chat_messages {
                                self.render_chat_message(ui, msg);
                                ui.add_space(12.0);
                            }
                        }

                        if self.chat_loading {
                            ui.horizontal(|ui| {
                                ui.add_space(8.0);
                                ui.spinner();
                                ui.label(RichText::new("Thinking...").size(13.0).color(TEXT_DIM));
                            });
                        }
                    });
            });

        ui.add_space(12.0);

        // Input area
        egui::Frame::none()
            .fill(INPUT_BG)
            .rounding(12.0)
            .inner_margin(12.0)
            .stroke(Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::multiline(&mut self.chat_input)
                        .hint_text("Ask about your vault content...")
                        .desired_width(ui.available_width() - 80.0)
                        .desired_rows(1)
                        .font(FontId::proportional(14.0));
                    
                    let response = ui.add(text_edit);
                    
                    let send_clicked = accent_btn(ui, "Send");
                    let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    
                    if (send_clicked || enter_pressed) && !self.chat_input.trim().is_empty() && !self.chat_loading {
                        let message = self.chat_input.trim().to_string();
                        self.chat_input.clear();
                        self.send_chat_message(vid, message, ctx);
                    }
                });
            });
    }

    fn render_chat_message(&self, ui: &mut Ui, msg: &ChatMessage) {
        let (bg_color, text_color, align) = if msg.role == "user" {
            (Color32::from_rgb(58, 44, 88), TEXT, egui::Align::Max)
        } else {
            (CARD, TEXT, egui::Align::Min)
        };

        ui.with_layout(egui::Layout::top_down(align), |ui| {
            egui::Frame::none()
                .fill(bg_color)
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() * 0.75);
                    
                    // Role label
                    let role_text = if msg.role == "user" { "You" } else { "Assistant" };
                    ui.label(RichText::new(role_text).size(11.0).color(TEXT_DIM).strong());
                    ui.add_space(4.0);
                    
                    // Message content
                    ui.label(RichText::new(&msg.content).size(14.0).color(text_color).line_height(Some(20.0)));
                    
                    // Referenced nodes badge
                    if !msg.referenced_nodes.is_empty() {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            badge(ui, &format!("📎 {} sources", msg.referenced_nodes.len()), ACCENT);
                        });
                    }
                    
                    // Timestamp
                    ui.add_space(4.0);
                    let time_str = msg.timestamp.format("%H:%M").to_string();
                    ui.label(RichText::new(time_str).size(10.0).color(TEXT_DIM));
                });
        });
    }

    fn send_chat_message(&mut self, vault_id: Uuid, message: String, ctx: &egui::Context) {
        // Add user message
        self.chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
            referenced_nodes: Vec::new(),
            timestamp: Utc::now(),
        });

        self.chat_loading = true;
        self.chat_referenced_nodes.clear();

        let rt = self.rt.clone();
        let ctx_clone = ctx.clone();
        let result_cell = Arc::new(Mutex::new(None));
        let result_cell_clone = result_cell.clone();

        std::thread::spawn(move || {
            rt.block_on(async move {
                let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
                let url = format!("{}/api/vaults/{}/chat", api_url, vault_id);
                
                let client = reqwest::Client::new();
                let payload = serde_json::json!({ "message": message });
                
                let result = match client.post(&url).json(&payload).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.json::<cofre_vault::models::ChatResponse>().await {
                                Ok(chat_response) => Ok(chat_response),
                                Err(e) => Err(format!("Failed to parse response: {}", e)),
                            }
                        } else {
                            Err(format!("API error: {}", response.status()))
                        }
                    }
                    Err(e) => Err(format!("Request failed: {}", e)),
                };

                *result_cell_clone.lock().unwrap() = Some(result);
                ctx_clone.request_repaint();
            });
        });

        // Poll for result
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        
        // Store the cell so we can poll it in update()
        self.chat_result_cell = Some(result_cell);
    }

    fn poll_chat_result(&mut self) {
        // Extract result without holding borrow
        let result_opt = if let Some(cell) = &self.chat_result_cell {
            if let Ok(mut guard) = cell.try_lock() {
                guard.take()
            } else {
                None
            }
        } else {
            None
        };

        // Process result
        if let Some(result) = result_opt {
            self.chat_loading = false;
            self.chat_result_cell = None;

            match result {
                Ok(response) => {
                    self.chat_referenced_nodes = response.referenced_node_ids.clone();
                    self.chat_messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: response.chat_reply_text,
                        referenced_nodes: response.referenced_node_ids,
                        timestamp: Utc::now(),
                    });
                    self.ok("Response received");
                }
                Err(e) => {
                    self.err(format!("Chat error: {}", e));
                }
            }
        }
    }
}
