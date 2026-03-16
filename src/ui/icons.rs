use egui_phosphor::regular as ph;

pub fn setup_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

pub const APP_LOGO: &str = ph::CODE;
pub const SETTINGS: &str = ph::GEAR;
pub const DOCS: &str = ph::BOOK_OPEN;
pub const FILE_OPEN: &str = ph::FOLDER_OPEN;
pub const FILE_SAVE: &str = ph::FLOPPY_DISK;
pub const FILE_SAVE_AS: &str = ph::FLOPPY_DISK_BACK;
pub const FILE_NEW: &str = ph::FILE_PLUS;
pub const RUN: &str = ph::PLAY;
pub const RUNNING: &str = ph::SPINNER;
pub const TAB_CLOSE: &str = ph::X;
pub const TAB_NEW: &str = ph::PLUS;
pub const DIRTY_DOT: &str = ph::CIRCLE;
pub const TERMINAL: &str = ph::TERMINAL_WINDOW;
pub const TERM_CLEAR: &str = ph::TRASH;
pub const TERM_COLLAPSE: &str = ph::CARET_DOWN;
pub const TERM_EXPAND: &str = ph::CARET_UP;
pub const UNSAVED: &str = ph::DOT_OUTLINE;
pub const ERROR: &str = ph::WARNING_CIRCLE;
pub const SUCCESS: &str = ph::CHECK_CIRCLE;
pub const SAVE_ACTION: &str = ph::FLOPPY_DISK;
pub const DISCARD: &str = ph::TRASH;
pub const CANCEL: &str = ph::X_CIRCLE;
pub const EMPTY_STATE: &str = ph::FILES;
pub const OPEN_FILE: &str = ph::FOLDER_OPEN;
pub const NEW_FILE: &str = ph::FILE_PLUS;
pub const DOC_CHAPTER: &str = ph::BOOK_BOOKMARK;
pub const CARET_UP: &str = ph::CARET_UP;
pub const CARET_DOWN: &str = ph::CARET_DOWN;
pub const CARET_RIGHT: &str = ph::CARET_RIGHT;
pub const MAGNIFY: &str = ph::MAGNIFYING_GLASS;
pub const ARROWS_CLOCKWISE: &str = ph::ARROWS_CLOCKWISE;
