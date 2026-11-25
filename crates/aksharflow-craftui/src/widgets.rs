use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use aksharflow_core::grapheme::GraphemeStatus;
use aksharflow_core::keymap::{FingerId, KeycapDescriptor, LayoutKind};
use craftui::prelude::*;
use craftui::text::TextEngine;
use kurbo::BezPath;

use crate::state::AppCore;

fn finger_color(f: FingerId) -> Color {
    match f {
        FingerId::LeftPinky | FingerId::RightPinky => Color::rgb(244, 63, 94),
        FingerId::LeftRing | FingerId::RightRing => Color::rgb(245, 158, 11),
        FingerId::LeftMiddle | FingerId::RightMiddle => Color::rgb(16, 185, 129),
        FingerId::LeftIndex => Color::rgb(14, 165, 233),
        FingerId::RightIndex => Color::rgb(99, 102, 241),
        FingerId::Thumb => Color::rgb(180, 83, 9),
    }
}

fn finger_label(f: FingerId, hi: bool) -> &'static str {
    match (f, hi) {
        (FingerId::LeftPinky, false) => "Left Pinky",
        (FingerId::LeftRing, false) => "Left Ring",
        (FingerId::LeftMiddle, false) => "Left Middle",
        (FingerId::LeftIndex, false) => "Left Index",
        (FingerId::Thumb, false) => "Thumb",
        (FingerId::RightIndex, false) => "Right Index",
        (FingerId::RightMiddle, false) => "Right Middle",
        (FingerId::RightRing, false) => "Right Ring",
        (FingerId::RightPinky, false) => "Right Pinky",
        (FingerId::LeftPinky, true) => "बायाँ कनिष्ठा",
        (FingerId::LeftRing, true) => "बायाँ अनामिका",
        (FingerId::LeftMiddle, true) => "बायाँ मध्यमा",
        (FingerId::LeftIndex, true) => "बायाँ तर्जनी",
        (FingerId::Thumb, true) => "अंगूठा",
        (FingerId::RightIndex, true) => "दायाँ तर्जनी",
        (FingerId::RightMiddle, true) => "दायाँ मध्यमा",
        (FingerId::RightRing, true) => "दायाँ अनामिका",
        (FingerId::RightPinky, true) => "दायाँ कनिष्ठा",
    }
}

/// Interactive grapheme-clustered typing stage receiving hardware & virtual keyboard input.
pub struct TypingStageWidget {
    pub core: Rc<RefCell<AppCore>>,
}

impl CustomWidget for TypingStageWidget {
    fn debug_name(&self) -> &'static str {
        "TypingStageWidget"
    }

    fn focusable(&self) -> bool {
        true
    }

    fn wants_tab(&self) -> bool {
        true
    }

    fn wants_key_up(&self) -> bool {
        true
    }

    fn autofocus(&self) -> bool {
        true
    }

    fn measure(&mut self, c: Constraints, _te: &mut TextEngine, _theme: &ThemeTokens) -> Size {
        Size::new(c.max_width.max(320.0), 196.0)
    }

    fn paint(
        &mut self,
        rect: Rect,
        scene: &mut PaintScene,
        text_engine: &mut TextEngine,
        theme: &ThemeTokens,
        focused: bool,
    ) {
        let Ok(c) = self.core.try_borrow() else {
            return;
        };
        let is_dark = theme.mode == ThemeMode::Dark || theme.mode == ThemeMode::Amoled;

        scene.fill_rect(rect, theme.surface, CornerRadii::all(14.0));
        scene.stroke_rect(
            rect,
            if focused { theme.primary } else { theme.border },
            if focused { 2.0 } else { 1.0 },
            CornerRadii::all(14.0),
        );

        let is_hindi = c.layout != LayoutKind::EnQwerty;
        let style_base = TextStyle::default()
            .with_family(if is_hindi {
                "Noto Sans Devanagari"
            } else {
                "monospace"
            })
            .with_size(if is_hindi { 27.0 } else { 23.0 })
            .with_weight(500.0);

        let pad_x = 24.0;
        let pad_y = 26.0;
        let max_x = rect.max_x() - pad_x;
        let line_h = if is_hindi { 48.0 } else { 40.0 };

        // Group graphemes into non-breaking words separated by spaces.
        let mut groups: Vec<Vec<(usize, &str)>> = Vec::new();
        let mut cur_word = Vec::new();
        for (i, g) in c.graphemes.iter().enumerate() {
            if g == " " {
                if !cur_word.is_empty() {
                    groups.push(std::mem::take(&mut cur_word));
                }
                groups.push(vec![(i, " ")]);
            } else {
                cur_word.push((i, g.as_str()));
            }
        }
        if !cur_word.is_empty() {
            groups.push(cur_word);
        }

        let matched_col = if is_dark {
            Color::rgb(52, 211, 153)
        } else {
            Color::rgb(4, 120, 87)
        };
        let error_col = if is_dark {
            Color::rgb(248, 113, 113)
        } else {
            Color::rgb(185, 28, 28)
        };
        let partial_col = if is_dark {
            Color::rgb(252, 211, 77)
        } else {
            Color::rgb(180, 83, 9)
        };
        let pending_col = theme.text_secondary;

        let mut x = rect.x + pad_x;
        let mut y = rect.y + pad_y;

        scene.push_clip(rect.inset(Insets::all(4.0)), CornerRadii::all(10.0));
        for group in groups {
            let word_text: String = group.iter().map(|(_, s)| *s).collect();
            let word_sz = if word_text == " " {
                Size::new(12.0, line_h)
            } else {
                text_engine.measure(&word_text, &style_base, None)
            };

            if x > rect.x + pad_x && x + word_sz.width > max_x {
                x = rect.x + pad_x;
                y += line_h;
            }

            for (idx, glyph) in group {
                let g_sz = if glyph == " " {
                    Size::new(11.0, line_h)
                } else {
                    text_engine.measure(glyph, &style_base, None)
                };
                let status = c.statuses.get(idx).copied().unwrap_or(GraphemeStatus::Pending);
                let is_active = idx == c.grapheme_index;

                if is_active {
                    let bg = if c.active_has_error {
                        error_col.with_alpha(0.18)
                    } else {
                        theme.primary.with_alpha(0.16)
                    };
                    scene.fill_rect(
                        Rect::new(x - 1.0, y, g_sz.width + 2.0, line_h - 8.0),
                        bg,
                        CornerRadii::all(4.0),
                    );
                    scene.fill_rect(
                        Rect::new(x - 1.5, y + 3.0, 2.5, line_h - 14.0),
                        theme.primary,
                        CornerRadii::all(2.0),
                    );

                    // Floating IME/partial typed preview chip above the active grapheme.
                    let preview = match (&c.pending_ime_preview, c.active_typed_preview.is_empty()) {
                        (Some(p), false) => Some(format!("{} + {p}", c.active_typed_preview)),
                        (Some(p), true) => Some(p.clone()),
                        (None, false) => Some(c.active_typed_preview.clone()),
                        (None, true) => None,
                    };
                    if let Some(prev) = preview {
                        let pst = TextStyle::default()
                            .with_family(if is_hindi {
                                "Noto Sans Devanagari"
                            } else {
                                "monospace"
                            })
                            .with_size(12.0)
                            .with_weight(700.0)
                            .with_color(theme.primary);
                        let psz = text_engine.measure(&prev, &pst, None);
                        let chip = Rect::new(x, y - 22.0, (psz.width + 10.0).max(20.0), 19.0);
                        scene.fill_rect(chip, theme.surface_variant, CornerRadii::all(4.0));
                        scene.stroke_rect(chip, theme.primary, 1.0, CornerRadii::all(4.0));
                        scene.draw_text(Point::new(chip.x + 5.0, chip.y + 2.0), &prev, pst, None);
                    }
                }

                let color = if is_active && c.active_has_error {
                    error_col
                } else if is_active && c.active_is_partial {
                    partial_col
                } else if is_active {
                    theme.text_primary
                } else {
                    match status {
                        GraphemeStatus::Matched => matched_col,
                        GraphemeStatus::Error => error_col,
                        GraphemeStatus::Pending => pending_col,
                    }
                };

                if status == GraphemeStatus::Error || (is_active && c.active_has_error) {
                    scene.fill_rect(
                        Rect::new(x, y + line_h - 11.0, g_sz.width, 2.0),
                        error_col,
                        CornerRadii::ZERO,
                    );
                }

                if glyph != " " {
                    scene.draw_text(
                        Point::new(x, y + 2.0),
                        glyph,
                        style_base.clone().with_color(color),
                        None,
                    );
                }
                x += g_sz.width + 1.0;
            }
        }
        scene.pop_clip();

        if !focused {
            scene.fill_rect(
                rect,
                theme.background.with_alpha(0.72),
                CornerRadii::all(14.0),
            );
            let prompt = if c.ui_lang_hi {
                "शुरू करने के लिए यहाँ क्लिक करें या टाइप करें"
            } else {
                "Click here or start typing to begin"
            };
            let pst = TextStyle::default()
                .with_size(14.0)
                .with_weight(600.0)
                .with_color(theme.text_primary);
            let psz = text_engine.measure(prompt, &pst, None);
            let pill = Rect::new(
                rect.center().x - (psz.width + 28.0) * 0.5,
                rect.center().y - 19.0,
                psz.width + 28.0,
                38.0,
            );
            scene.fill_rect(pill, theme.surface, CornerRadii::all(10.0));
            scene.stroke_rect(pill, theme.primary, 1.5, CornerRadii::all(10.0));
            scene.draw_text(Point::new(pill.x + 14.0, pill.y + 10.0), prompt, pst, None);
        }
    }

    fn handle_event(&mut self, _rect: Rect, event: &UiEvent) -> bool {
        if let UiEvent::Key(ke) = event
            && let Ok(mut c) = self.core.try_borrow_mut()
        {
            return c.handle_craft_typing_key(ke);
        }
        false
    }

    fn a11y(&self) -> A11yMetadata {
        A11yMetadata {
            role: A11yRole::TextInput,
            label: Some("Typing Stage".to_string()),
            focusable: true,
            ..Default::default()
        }
    }
}

/// Interactive touch-capable Virtual Keyboard showing next expected key, finger color zones,
/// and toggleable Red-to-Green adaptive confidence heatmap.
pub struct VirtualKeyboardWidget {
    pub core: Rc<RefCell<AppCore>>,
    key_rects: Vec<(String, Rect)>,
    heatmap_rect: Rect,
}

impl VirtualKeyboardWidget {
    pub fn new(core: Rc<RefCell<AppCore>>) -> Self {
        Self {
            core,
            key_rects: Vec::new(),
            heatmap_rect: Rect::ZERO,
        }
    }
}

impl CustomWidget for VirtualKeyboardWidget {
    fn debug_name(&self) -> &'static str {
        "VirtualKeyboardWidget"
    }

    fn measure(&mut self, c: Constraints, _te: &mut TextEngine, _theme: &ThemeTokens) -> Size {
        Size::new(c.max_width.max(460.0), 232.0)
    }

    fn paint(
        &mut self,
        rect: Rect,
        scene: &mut PaintScene,
        text_engine: &mut TextEngine,
        theme: &ThemeTokens,
        _focused: bool,
    ) {
        self.key_rects.clear();
        let Ok(c) = self.core.try_borrow() else {
            return;
        };
        let is_dark = theme.mode == ThemeMode::Dark || theme.mode == ThemeMode::Amoled;
        scene.fill_rect(rect, theme.surface, CornerRadii::all(14.0));
        scene.stroke_rect(rect, theme.border, 1.0, CornerRadii::all(14.0));

        // Top hint strip + Heatmap toggle pill
        let hint_text = if let Some(ref hint) = c.next_key_hint {
            let mut prefix = String::from("Next: ");
            if hint.shift {
                prefix.push_str("Shift + ");
            }
            if hint.alt_gr {
                prefix.push_str("AltGr + ");
            }
            format!("{prefix}{}", hint.expected_label)
        } else {
            "Tap any keycap to type".to_string()
        };
        let is_hindi = c.layout != LayoutKind::EnQwerty;
        let h_style = TextStyle::default()
            .with_family(if is_hindi {
                "Noto Sans Devanagari"
            } else {
                "sans-serif"
            })
            .with_size(12.0)
            .with_weight(600.0)
            .with_color(theme.text_primary);
        scene.draw_text(Point::new(rect.x + 12.0, rect.y + 8.0), &hint_text, h_style, None);

        let hm_rect = Rect::new(rect.max_x() - 108.0, rect.y + 5.0, 96.0, 22.0);
        self.heatmap_rect = hm_rect;
        scene.fill_rect(
            hm_rect,
            if c.show_heatmap {
                theme.primary_container
            } else {
                theme.surface_variant
            },
            CornerRadii::all(6.0),
        );
        scene.stroke_rect(hm_rect, theme.border, 1.0, CornerRadii::all(6.0));
        scene.draw_text(
            Point::new(hm_rect.x + 10.0, hm_rect.y + 4.0),
            if c.show_heatmap {
                "Heatmap: ON"
            } else {
                "Heatmap: OFF"
            },
            TextStyle::default()
                .with_size(11.0)
                .with_weight(600.0)
                .with_color(theme.text_primary),
            None,
        );

        let eff_shift = c.shift_held || c.touch_shift;
        let eff_alt_gr = c.alt_gr_held || c.touch_alt_gr;

        let mut stat_by_code: HashMap<&str, (f64, bool, bool)> = HashMap::new();
        for st in &c.key_stats {
            if st.requires_shift == eff_shift && st.requires_alt_gr == eff_alt_gr {
                stat_by_code.insert(
                    st.physical_code.as_str(),
                    (st.confidence_score, st.hits + st.misses > 0, st.unlocked),
                );
            }
        }

        let mut rows: [Vec<&KeycapDescriptor>; 5] = [vec![], vec![], vec![], vec![], vec![]];
        for cap in &c.keycaps {
            let r = cap.row as usize;
            if r < 5 {
                rows[r].push(cap);
            }
        }

        let avail_w = (rect.width - 24.0).max(200.0);
        let row_h = 34.0;
        let row_gap = 4.0;
        let start_y = rect.y + 34.0;

        for (r_idx, row) in rows.iter().enumerate() {
            if row.is_empty() {
                continue;
            }
            let total_units: f32 = row.iter().map(|k| k.width_u as f32).sum();
            let gap = 4.0;
            let unit_px = ((avail_w - gap * (row.len().saturating_sub(1) as f32)) / total_units.max(1.0)).max(12.0);
            let mut kx = rect.x + 12.0;
            let ky = start_y + (r_idx as f32) * (row_h + row_gap);

            for cap in row {
                let kw = cap.width_u as f32 * unit_px;
                let krect = Rect::new(kx, ky, kw, row_h);
                self.key_rects.push((cap.code.clone(), krect));

                let is_expected = c.next_key_hint.as_ref().is_some_and(|h| {
                    h.code == cap.code
                        || (h.shift && (cap.code == "ShiftLeft" || cap.code == "ShiftRight"))
                        || (h.alt_gr && cap.code == "AltRight")
                });

                let mut bg = theme.background;
                if c.show_heatmap
                    && let Some(&(conf, practiced, unlocked)) = stat_by_code.get(cap.code.as_str())
                    && unlocked
                    && practiced
                {
                    bg = if conf >= 85.0 {
                        Color::rgba(16, 185, 129, if is_dark { 65 } else { 45 })
                    } else if conf >= 55.0 {
                        Color::rgba(245, 158, 11, if is_dark { 65 } else { 45 })
                    } else {
                        Color::rgba(239, 68, 68, if is_dark { 75 } else { 50 })
                    };
                }
                if is_expected {
                    bg = theme.primary.with_alpha(0.24);
                }

                scene.fill_rect(krect, bg, CornerRadii::all(6.0));
                scene.stroke_rect(
                    krect,
                    if is_expected {
                        theme.primary
                    } else {
                        theme.border
                    },
                    if is_expected { 2.0 } else { 1.0 },
                    CornerRadii::all(6.0),
                );
                // Bottom finger color zone indicator
                scene.fill_rect(
                    Rect::new(krect.x + 3.0, krect.max_y() - 3.0, (krect.width - 6.0).max(2.0), 2.5),
                    finger_color(cap.finger),
                    CornerRadii::all(1.5),
                );

                let label = if eff_alt_gr {
                    cap.alt_gr_label.as_deref().unwrap_or(&cap.normal_label)
                } else if eff_shift {
                    &cap.shift_label
                } else {
                    &cap.normal_label
                };
                let lst = TextStyle::default()
                    .with_family(if is_hindi {
                        "Noto Sans Devanagari"
                    } else {
                        "sans-serif"
                    })
                    .with_size(if is_hindi && label.chars().count() <= 2 {
                        14.0
                    } else {
                        11.5
                    })
                    .with_weight(if is_expected { 700.0 } else { 500.0 })
                    .with_color(theme.text_primary);
                let lsz = text_engine.measure(label, &lst, None);
                scene.draw_text(
                    Point::new(
                        krect.center().x - lsz.width * 0.5,
                        krect.center().y - lsz.height * 0.5 - 1.0,
                    ),
                    label,
                    lst,
                    None,
                );

                kx += kw + gap;
            }
        }
    }

    fn handle_event(&mut self, _rect: Rect, event: &UiEvent) -> bool {
        if let UiEvent::Pointer(pe) = event
            && matches!(pe.phase, PointerPhase::Down)
        {
            if self.heatmap_rect.contains(pe.position)
                && let Ok(mut c) = self.core.try_borrow_mut()
            {
                c.show_heatmap = !c.show_heatmap;
                c.bump();
                return true;
            }
            for (code, r) in &self.key_rects {
                if r.contains(pe.position)
                    && let Ok(mut c) = self.core.try_borrow_mut()
                {
                    c.press_virtual_key(code);
                    return true;
                }
            }
        }
        false
    }
}

/// Compact dual-hand 10-finger posture guide highlighting which finger strikes the next expected key.
pub struct FingerGuideWidget {
    pub core: Rc<RefCell<AppCore>>,
}

impl CustomWidget for FingerGuideWidget {
    fn debug_name(&self) -> &'static str {
        "FingerGuideWidget"
    }

    fn measure(&mut self, c: Constraints, _te: &mut TextEngine, _theme: &ThemeTokens) -> Size {
        Size::new(240.0_f32.min(c.max_width.max(180.0)), 232.0)
    }

    fn paint(
        &mut self,
        rect: Rect,
        scene: &mut PaintScene,
        _text_engine: &mut TextEngine,
        theme: &ThemeTokens,
        _focused: bool,
    ) {
        let Ok(c) = self.core.try_borrow() else {
            return;
        };
        scene.fill_rect(rect, theme.surface, CornerRadii::all(14.0));
        scene.stroke_rect(rect, theme.border, 1.0, CornerRadii::all(14.0));

        let title = if c.ui_lang_hi { "उंगली गाइड" } else { "Finger Guide" };
        scene.draw_text(
            Point::new(rect.x + 12.0, rect.y + 10.0),
            title,
            TextStyle::default()
                .with_size(12.0)
                .with_weight(700.0)
                .with_color(theme.text_primary),
            None,
        );

        let active_finger = c.next_key_hint.as_ref().map(|h| h.finger);
        let needs_shift = c.next_key_hint.as_ref().is_some_and(|h| h.shift);

        let fingers = [
            (FingerId::LeftPinky, 0.10, 0.36, 40.0),
            (FingerId::LeftRing, 0.19, 0.28, 52.0),
            (FingerId::LeftMiddle, 0.28, 0.23, 58.0),
            (FingerId::LeftIndex, 0.37, 0.29, 50.0),
            (FingerId::Thumb, 0.46, 0.52, 34.0),
            (FingerId::RightIndex, 0.60, 0.29, 50.0),
            (FingerId::RightMiddle, 0.69, 0.23, 58.0),
            (FingerId::RightRing, 0.78, 0.28, 52.0),
            (FingerId::RightPinky, 0.87, 0.36, 40.0),
        ];

        // Palms
        let left_palm = Rect::new(rect.x + 18.0, rect.y + 122.0, 82.0, 52.0);
        let right_palm = Rect::new(rect.max_x() - 100.0, rect.y + 122.0, 82.0, 52.0);
        scene.fill_rect(left_palm, theme.surface_variant, CornerRadii::all(16.0));
        scene.fill_rect(right_palm, theme.surface_variant, CornerRadii::all(16.0));

        for (fid, xr, yr, fh) in fingers {
            let fx = rect.x + (rect.width * xr as f32) - 8.0;
            let fy = rect.y + (rect.height * yr as f32);
            let active = active_finger == Some(fid)
                || (needs_shift && matches!(fid, FingerId::LeftPinky | FingerId::RightPinky));
            let fr = Rect::new(fx, fy, 16.0, fh);
            let fcol = finger_color(fid);
            scene.fill_rect(
                fr,
                if active { fcol } else { theme.surface_variant },
                CornerRadii::all(8.0),
            );
            scene.stroke_rect(fr, fcol, if active { 2.0 } else { 1.0 }, CornerRadii::all(8.0));
        }

        if let Some(f) = active_finger {
            let mut label = finger_label(f, c.ui_lang_hi).to_string();
            if needs_shift {
                label.push_str(" + Shift");
            }
            scene.draw_text(
                Point::new(rect.x + 12.0, rect.max_y() - 28.0),
                &label,
                TextStyle::default()
                    .with_family(if c.ui_lang_hi {
                        "Noto Sans Devanagari"
                    } else {
                        "sans-serif"
                    })
                    .with_size(12.0)
                    .with_weight(700.0)
                    .with_color(finger_color(f)),
                None,
            );
        }
    }
}

/// 60 FPS Canvas Balloon Pop Mini-Game routed directly through Rust's IME pipeline.
pub struct BalloonMiniGameWidget {
    pub core: Rc<RefCell<AppCore>>,
    btn_back: Rect,
    btn_pause: Rect,
    btn_restart: Rect,
    btn_finish: Rect,
}

impl BalloonMiniGameWidget {
    pub fn new(core: Rc<RefCell<AppCore>>) -> Self {
        Self {
            core,
            btn_back: Rect::ZERO,
            btn_pause: Rect::ZERO,
            btn_restart: Rect::ZERO,
            btn_finish: Rect::ZERO,
        }
    }
}

impl CustomWidget for BalloonMiniGameWidget {
    fn debug_name(&self) -> &'static str {
        "BalloonMiniGameWidget"
    }

    fn focusable(&self) -> bool {
        true
    }

    fn wants_key_up(&self) -> bool {
        true
    }

    fn autofocus(&self) -> bool {
        true
    }

    fn measure(&mut self, c: Constraints, _te: &mut TextEngine, _theme: &ThemeTokens) -> Size {
        Size::new(c.max_width.max(400.0), 420.0)
    }

    fn paint(
        &mut self,
        rect: Rect,
        scene: &mut PaintScene,
        text_engine: &mut TextEngine,
        theme: &ThemeTokens,
        focused: bool,
    ) {
        if let Ok(mut c) = self.core.try_borrow_mut() {
            c.balloon.canvas_width = rect.width;
            c.balloon.canvas_height = rect.height - 86.0;
        }
        let Ok(c) = self.core.try_borrow() else {
            return;
        };

        let is_dark = theme.mode == ThemeMode::Dark || theme.mode == ThemeMode::Amoled;
        let is_hindi = c.layout != LayoutKind::EnQwerty;

        scene.fill_rect(rect, theme.surface, CornerRadii::all(14.0));
        scene.stroke_rect(
            rect,
            if focused { theme.primary } else { theme.border },
            if focused { 2.0 } else { 1.0 },
            CornerRadii::all(14.0),
        );

        // Top HUD controls
        let header_y = rect.y + 10.0;
        self.btn_back = Rect::new(rect.x + 12.0, header_y, 98.0, 30.0);
        self.btn_pause = Rect::new(rect.max_x() - 284.0, header_y, 82.0, 30.0);
        self.btn_restart = Rect::new(rect.max_x() - 194.0, header_y, 88.0, 30.0);
        self.btn_finish = Rect::new(rect.max_x() - 98.0, header_y, 86.0, 30.0);

        for (btn, label, primary) in [
            (self.btn_back, "Back Map", false),
            (
                self.btn_pause,
                if c.balloon.paused { "Resume" } else { "Pause" },
                false,
            ),
            (self.btn_restart, "Restart", false),
            (self.btn_finish, "Finish", true),
        ] {
            scene.fill_rect(
                btn,
                if primary {
                    theme.primary
                } else {
                    theme.surface_variant
                },
                CornerRadii::all(8.0),
            );
            scene.stroke_rect(btn, theme.border, 1.0, CornerRadii::all(8.0));
            scene.draw_text(
                Point::new(btn.x + 12.0, btn.y + 8.0),
                label,
                TextStyle::default()
                    .with_size(12.0)
                    .with_weight(600.0)
                    .with_color(if primary {
                        theme.on_primary
                    } else {
                        theme.text_primary
                    }),
                None,
            );
        }

        let stats = format!(
            "Score: {}   Popped: {}/{}   Missed: {}   Lives: {}   WPM: {}",
            c.balloon.score,
            c.balloon.popped,
            c.balloon.words.len(),
            c.balloon.missed,
            c.balloon.lives,
            c.metrics.net_wpm.round() as i32
        );
        scene.draw_text(
            Point::new(rect.x + 122.0, header_y + 8.0),
            &stats,
            TextStyle::default()
                .with_size(12.5)
                .with_weight(600.0)
                .with_color(theme.text_primary),
            Some((rect.width - 420.0).max(120.0)),
        );

        // Arena box
        let arena = Rect::new(rect.x + 12.0, rect.y + 48.0, rect.width - 24.0, rect.height - 90.0);
        scene.fill_rect(arena, theme.background, CornerRadii::all(10.0));
        scene.stroke_rect(arena, theme.border, 1.0, CornerRadii::all(10.0));

        scene.push_clip(arena, CornerRadii::all(10.0));
        // Floor line
        let floor_y = arena.max_y() - 20.0;
        let mut floor_path = BezPath::new();
        floor_path.move_to(((arena.x + 14.0) as f64, floor_y as f64));
        floor_path.line_to(((arena.max_x() - 14.0) as f64, floor_y as f64));
        scene.stroke_path(floor_path, theme.border, 1.5);

        let w_style = TextStyle::default()
            .with_family(if is_hindi {
                "Noto Sans Devanagari"
            } else {
                "monospace"
            })
            .with_size(if is_hindi { 18.0 } else { 16.0 })
            .with_weight(700.0);

        for b in &c.balloon.balloons {
            let cx = arena.x + arena.width * b.x_ratio;
            let cy = arena.y + b.y_px;
            let is_locked = c.balloon.locked_word.as_deref() == Some(b.word.as_str());
            let sz = text_engine.measure(&b.word, &w_style, None);
            let pill_w = (sz.width + 28.0).max(68.0);
            let pill_h = 36.0;

            let mut str_path = BezPath::new();
            str_path.move_to((cx as f64, (cy + pill_h * 0.5) as f64));
            str_path.line_to((cx as f64, (cy + pill_h * 0.5 + 12.0) as f64));
            scene.stroke_path(str_path, theme.text_secondary, 1.0);

            let capsule = Rect::new(cx - pill_w * 0.5, cy - pill_h * 0.5, pill_w, pill_h);
            scene.fill_rect(
                capsule,
                if is_locked {
                    if is_dark {
                        Color::rgb(69, 26, 3)
                    } else {
                        Color::rgb(254, 243, 199)
                    }
                } else {
                    theme.surface
                },
                CornerRadii::all(18.0),
            );
            scene.stroke_rect(
                capsule,
                if is_locked { theme.primary } else { theme.border },
                if is_locked { 2.5 } else { 1.5 },
                CornerRadii::all(18.0),
            );
            scene.draw_text(
                Point::new(cx - sz.width * 0.5, cy - sz.height * 0.5),
                &b.word,
                w_style.clone().with_color(if is_locked {
                    if is_dark {
                        Color::rgb(252, 211, 77)
                    } else {
                        Color::rgb(146, 64, 14)
                    }
                } else {
                    theme.text_primary
                }),
                None,
            );
        }

        for p in &c.balloon.particles {
            scene.fill_rect(
                Rect::new(arena.x + p.x - 3.0, arena.y + p.y - 3.0, 6.0, 6.0),
                theme.primary.with_alpha(p.life.clamp(0.0, 1.0)),
                CornerRadii::all(3.0),
            );
        }
        scene.pop_clip();

        // Bottom buffer & hint readout
        let locked = c.balloon.locked_word.as_deref().unwrap_or("--");
        let buf = if !c.balloon.typed_preview.is_empty() {
            c.balloon.typed_preview.as_str()
        } else {
            c.pending_ime_preview.as_deref().unwrap_or("--")
        };
        let footer = format!(
            "Target: {locked}   |   Buffer: {buf}   |   Esc unlocks target, Space flushes Remington short-i"
        );
        scene.draw_text(
            Point::new(rect.x + 14.0, rect.max_y() - 30.0),
            &footer,
            TextStyle::default()
                .with_family(if is_hindi {
                    "Noto Sans Devanagari"
                } else {
                    "sans-serif"
                })
                .with_size(12.5)
                .with_weight(600.0)
                .with_color(if c.balloon.input_error {
                    theme.danger
                } else {
                    theme.text_primary
                }),
            None,
        );
    }

    fn handle_event(&mut self, _rect: Rect, event: &UiEvent) -> bool {
        match event {
            UiEvent::Pointer(pe) if matches!(pe.phase, PointerPhase::Down) => {
                if self.btn_back.contains(pe.position)
                    && let Ok(mut c) = self.core.try_borrow_mut()
                {
                    c.back_to_jungle_map();
                    return true;
                }
                if self.btn_pause.contains(pe.position)
                    && let Ok(mut c) = self.core.try_borrow_mut()
                {
                    c.balloon.paused = !c.balloon.paused;
                    c.bump();
                    return true;
                }
                if self.btn_restart.contains(pe.position)
                    && let Ok(mut c) = self.core.try_borrow_mut()
                {
                    c.restart_or_exit();
                    return true;
                }
                if self.btn_finish.contains(pe.position)
                    && let Ok(mut c) = self.core.try_borrow_mut()
                {
                    c.finish_balloon_wave();
                    return true;
                }
            }
            UiEvent::Key(ke) => {
                if let Ok(mut c) = self.core.try_borrow_mut() {
                    return c.handle_craft_game_key(ke);
                }
            }
            _ => {}
        }
        false
    }

    fn a11y(&self) -> A11yMetadata {
        A11yMetadata {
            role: A11yRole::Canvas,
            label: Some("Balloon Pop Mini-Game".to_string()),
            focusable: true,
            ..Default::default()
        }
    }
}

/// Custom vector chart plotting recent session Net WPM (Amber) and Accuracy % (Emerald).
pub struct WpmTrendChartWidget {
    pub core: Rc<RefCell<AppCore>>,
}

impl CustomWidget for WpmTrendChartWidget {
    fn debug_name(&self) -> &'static str {
        "WpmTrendChartWidget"
    }

    fn measure(&mut self, c: Constraints, _te: &mut TextEngine, _theme: &ThemeTokens) -> Size {
        Size::new(c.max_width.max(300.0), 170.0)
    }

    fn paint(
        &mut self,
        rect: Rect,
        scene: &mut PaintScene,
        _text_engine: &mut TextEngine,
        theme: &ThemeTokens,
        _focused: bool,
    ) {
        let Ok(c) = self.core.try_borrow() else {
            return;
        };
        scene.fill_rect(rect, theme.surface, CornerRadii::all(12.0));
        scene.stroke_rect(rect, theme.border, 1.0, CornerRadii::all(12.0));
        scene.draw_text(
            Point::new(rect.x + 14.0, rect.y + 10.0),
            "WPM & Accuracy Trend",
            TextStyle::default()
                .with_size(13.0)
                .with_weight(700.0)
                .with_color(theme.text_primary),
            None,
        );

        let sessions = c
            .analytics
            .as_ref()
            .map(|a| a.recent_sessions.as_slice())
            .unwrap_or(&[]);
        if sessions.is_empty() {
            scene.draw_text(
                Point::new(rect.x + 14.0, rect.center().y - 8.0),
                "Complete a typing line or exam session to view WPM & Accuracy trends.",
                TextStyle::default()
                    .with_size(12.0)
                    .with_color(theme.text_secondary),
                None,
            );
            return;
        }

        let plot = Rect::new(rect.x + 20.0, rect.y + 36.0, rect.width - 40.0, rect.height - 52.0);
        for step in 0..=4 {
            let gy = plot.y + (plot.height * step as f32) / 4.0;
            let mut line = BezPath::new();
            line.move_to((plot.x as f64, gy as f64));
            line.line_to((plot.max_x() as f64, gy as f64));
            scene.stroke_path(line, theme.border.with_alpha(0.55), 1.0);
        }

        let chronological: Vec<_> = sessions.iter().take(20).rev().collect();
        let max_wpm = chronological
            .iter()
            .map(|s| s.net_wpm as f32)
            .fold(40.0f32, f32::max)
            .max(40.0);

        let count = chronological.len();
        let mut wpm_path = BezPath::new();
        let mut acc_path = BezPath::new();

        for (i, sess) in chronological.iter().enumerate() {
            let px = if count <= 1 {
                plot.center().x
            } else {
                plot.x + (plot.width * i as f32) / ((count - 1) as f32)
            };
            let wy = plot.max_y() - ((sess.net_wpm as f32 / max_wpm).clamp(0.0, 1.0) * plot.height);
            let ay = plot.max_y() - ((sess.accuracy as f32 / 100.0).clamp(0.0, 1.0) * plot.height);
            if i == 0 {
                wpm_path.move_to((px as f64, wy as f64));
                acc_path.move_to((px as f64, ay as f64));
            } else {
                wpm_path.line_to((px as f64, wy as f64));
                acc_path.line_to((px as f64, ay as f64));
            }
        }
        scene.stroke_path(acc_path, Color::rgb(16, 185, 129), 2.0);
        scene.stroke_path(wpm_path, theme.primary, 2.5);
    }
}
