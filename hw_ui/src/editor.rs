//! Code editor component with syntax highlighting

use crate::{card_frame, header_text, body_text, AppTheme};
use eframe::egui;
use syntect::parsing::SyntaxSet;

pub struct CodeEditor {
    pub code: String,
    pub file_path: Option<String>,
    pub language: String,
    pub modified: bool,
    pub cursor_pos: usize,
    pub selection: Option<egui::text::CursorRange>,
    pub font_size: f32,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    
    // Syntax highlighting (simplified for now)
    _syntax_set: SyntaxSet,
}

impl CodeEditor {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        
        Self {
            code: String::new(),
            file_path: None,
            language: "cpp".to_string(),
            modified: false,
            cursor_pos: 0,
            selection: None,
            font_size: 14.0,
            show_line_numbers: true,
            word_wrap: false,
            _syntax_set: syntax_set,
        }
    }

    pub fn new_with_code(code: String, language: String) -> Self {
        let mut editor = Self::new();
        editor.code = code;
        editor.language = language;
        editor
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &AppTheme) -> bool {
        let mut changed = false;
        
        // Editor header
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            
            // File name or editor label
            let label_text = if let Some(ref path) = self.file_path {
                format!("📄 {}", std::path::Path::new(path).file_name().unwrap().to_string_lossy())
            } else {
                format!("📝 New File ({})", self.language.to_uppercase())
            };
            
            if self.modified {
                ui.label(crate::header_text(&format!("{} *", label_text)));
            } else {
                ui.label(header_text(&label_text));
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Language selector
                egui::ComboBox::from_label("")
                    .selected_text(self.language.to_uppercase())
                    .width(60.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.language, "cpp".to_string(), "C++");
                        ui.selectable_value(&mut self.language, "c".to_string(), "C");
                        ui.selectable_value(&mut self.language, "rust".to_string(), "Rust");
                        ui.selectable_value(&mut self.language, "python".to_string(), "Python");
                        ui.selectable_value(&mut self.language, "text".to_string(), "Plain Text");
                    });
                
                ui.separator();
                
                // Options
                ui.checkbox(&mut self.show_line_numbers, "Line Numbers");
                ui.checkbox(&mut self.word_wrap, "Word Wrap");
                
                ui.separator();
                
                // Actions
                if ui.button("💾 Save").clicked() {
                    changed = true;
                }
            });
        });

        ui.add_space(8.0);

        // Editor area
        let frame = card_frame(1.0);
        frame.show(ui, |ui| {
            // Calculate line numbers width
            let line_count = self.code.lines().count();
            let line_numbers_width = if self.show_line_numbers {
                format!("{}", line_count).len() as f32 * 8.0 + 16.0
            } else {
                0.0
            };

            // Main editor
            let available_width = ui.available_width() - line_numbers_width;
            let desired_height = ui.available_height() - 20.0;

            let response = ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .desired_width(available_width)
                    .code_editor()
                    .lock_focus(true)
            );

            // Handle cursor position and selection
            if response.changed() {
                self.modified = true;
                changed = true;
            }

            // Show line numbers if enabled
            if self.show_line_numbers {
                let line_numbers = self.generate_line_numbers();
                ui.painter().text(
                    egui::pos2(ui.cursor().min.x + 8.0, ui.cursor().min.y + 8.0),
                    egui::Align2::LEFT_TOP,
                    line_numbers,
                    egui::FontId::monospace(self.font_size),
                    theme.text_secondary,
                );
            }
        });

        // Status bar
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            
            // Cursor position
            let (line, col) = self.get_cursor_position();
            ui.label(body_text(&format!("Line: {}, Col: {}", line, col)));
            
            // Character count
            let line_count = self.code.lines().count();
            ui.label(body_text(&format!("Chars: {}, Lines: {}", self.code.len(), line_count)));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.modified {
                    ui.label(crate::warning_text("Unsaved changes"));
                } else {
                    ui.label(crate::success_text("Saved"));
                }
            });
        });

        changed
    }

    fn generate_line_numbers(&self) -> String {
        let line_count = self.code.lines().count();
        let mut line_numbers = String::new();
        
        for i in 1..=line_count {
            line_numbers.push_str(&format!("{}\n", i));
        }
        
        line_numbers
    }

    fn get_cursor_position(&self) -> (usize, usize) {
        let bytes_before_cursor = &self.code[..self.cursor_pos.min(self.code.len())];
        let line = bytes_before_cursor.lines().count();
        let col = bytes_before_cursor.lines().last().map(|l| l.len()).unwrap_or(0);
        
        (line, col)
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.code = content;
        self.file_path = Some(path.to_string());
        self.modified = false;
        
        // Detect language from file extension
        self.language = self.detect_language_from_path(path);
        
        Ok(())
    }

    pub fn save_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref path) = self.file_path {
            std::fs::write(path, &self.code)?;
            Ok(())
        } else {
            Err("No file path specified".into())
        }
    }

    pub fn save_file_as(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, &self.code)?;
        self.file_path = Some(path.to_string());
        self.modified = false;
        self.language = self.detect_language_from_path(path);
        Ok(())
    }

    fn detect_language_from_path(&self, path: &str) -> String {
        if let Some(extension) = std::path::Path::new(path).extension() {
            match extension.to_string_lossy().as_ref() {
                "cpp" | "cxx" | "cc" | "c++" => "cpp".to_string(),
                "c" | "h" => "c".to_string(),
                "rs" => "rust".to_string(),
                "py" => "python".to_string(),
                "txt" => "text".to_string(),
                _ => "text".to_string(),
            }
        } else {
            "text".to_string()
        }
    }

    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    pub fn insert_text(&mut self, text: &str) {
        let cursor_pos = self.cursor_pos.min(self.code.len());
        self.code.insert_str(cursor_pos, text);
        self.cursor_pos += text.len();
        self.modified = true;
    }

    pub fn get_selected_text(&self) -> Option<String> {
        if let Some(ref selection) = self.selection {
            // Use the primary cursor for selection
            let start = selection.primary.ccursor.index.min(selection.secondary.ccursor.index);
            let end = selection.primary.ccursor.index.max(selection.secondary.ccursor.index);
            if start < end {
                Some(self.code[start..end].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn find_and_replace(&mut self, find: &str, replace: &str, replace_all: bool) -> usize {
        if find.is_empty() {
            return 0;
        }

        let mut replacements = 0;
        
        if replace_all {
            self.code = self.code.replace(find, replace);
            replacements = self.code.matches(find).count();
        } else {
            if let Some(pos) = self.code.find(find) {
                self.code.replace_range(pos..pos + find.len(), replace);
                replacements = 1;
            }
        }

        if replacements > 0 {
            self.modified = true;
        }

        replacements
    }

    pub fn get_word_at_cursor(&self) -> Option<String> {
        let cursor_pos = self.cursor_pos.min(self.code.len());
        
        // Find word boundaries
        let start = self.code[..cursor_pos].rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1).unwrap_or(0);
        let end = self.code[cursor_pos..].find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| cursor_pos + i).unwrap_or(self.code.len());
        
        if start < end {
            Some(self.code[start..end].to_string())
        } else {
            None
        }
    }
}
