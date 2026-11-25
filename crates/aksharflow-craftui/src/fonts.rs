use craftui::prelude::*;

pub const NOTO_DEVANAGARI_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/NotoSansDevanagari-Regular.ttf");
pub const NOTO_DEVANAGARI_BOLD: &[u8] =
    include_bytes!("../assets/fonts/NotoSansDevanagari-Bold.ttf");
pub const VERA_MONO: &[u8] = include_bytes!("../assets/fonts/VeraMono.ttf");

/// Registers bundled Devanagari and monospace fonts on the CraftUI `WidgetTree`
/// so Hindi conjuncts, matras, and tabular metrics render consistently on Desktop and Web WASM.
pub fn register_bundled_fonts(tree: &mut WidgetTree) {
    tree.register_fallback_font_bytes(NOTO_DEVANAGARI_REGULAR.to_vec());
    tree.register_fallback_font_bytes(NOTO_DEVANAGARI_BOLD.to_vec());
    tree.register_monospace_font_bytes(VERA_MONO.to_vec());
}
