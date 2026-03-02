use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::ui::editor::CodeEditor;
use crate::ui::theme::Theme;

pub struct Tab {
    pub code: String,
    pub last_saved_code: String,
    pub current_file: Option<PathBuf>,
    pub editor: CodeEditor,
    pub output_rx: Option<Arc<Mutex<Vec<String>>>>,
    pub is_running: bool,
    /// Unique stable ID used to give each tab's TextEdit its own undo history.
    pub id: usize,
}

static TAB_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Tab {
    fn next_id() -> usize {
        TAB_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn new(theme: Theme) -> Self {
        let code = String::from("!start\n# code here\n!end\n");
        Self {
            last_saved_code: code.clone(),
            code,
            current_file: None,
            editor: CodeEditor::new(theme),
            output_rx: None,
            is_running: false,
            id: Self::next_id(),
        }
    }

    pub fn from_file(path: PathBuf, content: String, theme: Theme) -> Self {
        Self {
            last_saved_code: content.clone(),
            code: content,
            current_file: Some(path),
            editor: CodeEditor::new(theme),
            output_rx: None,
            is_running: false,
            id: Self::next_id(),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.code != self.last_saved_code
    }

    pub fn display_name(&self) -> String {
        self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn is_pristine_new(&self) -> bool {
        self.current_file.is_none() && !self.is_dirty()
    }
}