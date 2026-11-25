use std::cell::RefCell;
use std::rc::Rc;

use aksharflow_core::session::{PracticeMode, SessionEngine};
use craftui::prelude::*;
use craftui::reactive::mark_node_dirty;

use crate::fonts::register_bundled_fonts;
use crate::state::{AppCore, StudioTab, aot_ui};
use crate::widgets::{
    BalloonMiniGameWidget, FingerGuideWidget, TypingStageWidget, VirtualKeyboardWidget,
    WpmTrendChartWidget,
};

fn build_akshar_theme(dark: bool) -> ThemeTokens {
    let mut t = if dark {
        ThemeTokens::dark()
    } else {
        ThemeTokens::light()
    };
    if dark {
        t.background = Color::rgb(15, 23, 42);
        t.surface = Color::rgb(30, 41, 59);
        t.surface_variant = Color::rgb(51, 65, 85);
        t.primary = Color::rgb(245, 158, 11);
        t.on_primary = Color::rgb(15, 23, 42);
        t.primary_container = Color::rgb(69, 26, 3);
        t.text_primary = Color::rgb(241, 245, 249);
        t.text_secondary = Color::rgb(148, 163, 184);
        t.border = Color::rgb(71, 85, 105);
        t.danger = Color::rgb(248, 113, 113);
        t.on_error = Color::rgb(15, 23, 42);
    } else {
        t.background = Color::rgb(248, 250, 252);
        t.surface = Color::WHITE;
        t.surface_variant = Color::rgb(241, 245, 249);
        t.primary = Color::rgb(180, 83, 9);
        t.on_primary = Color::WHITE;
        t.primary_container = Color::rgb(254, 243, 199);
        t.text_primary = Color::rgb(15, 23, 42);
        t.text_secondary = Color::rgb(71, 85, 105);
        t.border = Color::rgb(203, 213, 225);
        t.danger = Color::rgb(185, 28, 28);
        t.on_error = Color::WHITE;
    }
    t
}

/// Mounts the complete AksharFlow v2.0.0 CraftUI application into `tree` and returns the root `NodeId`
/// plus the shared reactive `AppCore` handle.
pub fn mount_aksharflow_app(
    cx: Scope,
    tree: &mut WidgetTree,
    engine: SessionEngine,
) -> (NodeId, Rc<RefCell<AppCore>>) {
    register_bundled_fonts(tree);

    let revision = cx.signal(0u64);
    let tab_sig = cx.signal(StudioTab::Adaptive as usize);
    let zen_sig = cx.signal(false);
    let mode_sig = cx.signal(PracticeMode::Adaptive);
    let has_active_lesson_sig = cx.signal(false);

    let top_bar = aot_ui::AksharTopBar::new(cx);
    let metrics_bar = aot_ui::AksharMetricsBar::new(cx);
    let unlock_strip = aot_ui::AksharUnlockStrip::new(cx);
    let jungle_panel = aot_ui::AksharJunglePanel::new(cx);
    let exam_bar = aot_ui::AksharExamBar::new(cx);
    let analytics_panel = aot_ui::AksharAnalyticsPanel::new(cx);
    let modals = aot_ui::AksharModalsOverlay::new(cx);

    let top_bar_node = top_bar.build(cx, tree);
    let metrics_node = metrics_bar.build(cx, tree);
    let unlock_node = unlock_strip.build(cx, tree);
    let jungle_node = jungle_panel.build(cx, tree);
    let exam_node = exam_bar.build(cx, tree);
    let analytics_node = analytics_panel.build(cx, tree);
    let modals_node = modals.build(cx, tree);

    let theme_sig = tree.theme_signal;
    let core = Rc::new(RefCell::new(AppCore::new(
        engine,
        revision,
        tab_sig,
        zen_sig,
        mode_sig,
        has_active_lesson_sig,
        top_bar.clone(),
        metrics_bar.clone(),
        unlock_strip.clone(),
        jungle_panel.clone(),
        exam_bar.clone(),
        analytics_panel.clone(),
        modals.clone(),
    )));

    {
        let initial_dark = core.borrow().theme_dark;
        tree.set_theme(build_akshar_theme(initial_dark));
        core.borrow_mut().theme_hook = Some(Rc::new(move |dark| {
            theme_sig.set(build_akshar_theme(dark));
        }));
    }

    // Wire declarative .cui callbacks -> AppCore actions
    top_bar.set_on_select_tab({
        let c = Rc::clone(&core);
        move |idx| c.borrow_mut().select_tab(idx.max(0) as usize)
    });
    top_bar.set_on_select_layout({
        let c = Rc::clone(&core);
        move |idx| c.borrow_mut().select_layout(idx.max(0) as usize)
    });
    top_bar.set_on_cycle_sound({
        let c = Rc::clone(&core);
        move || c.borrow_mut().cycle_sound()
    });
    top_bar.set_on_toggle_lang({
        let c = Rc::clone(&core);
        move || c.borrow_mut().toggle_lang()
    });
    top_bar.set_on_toggle_theme({
        let c = Rc::clone(&core);
        move || c.borrow_mut().toggle_theme()
    });
    top_bar.set_on_toggle_zen({
        let c = Rc::clone(&core);
        move || c.borrow_mut().toggle_zen()
    });
    {
        let m = modals.clone();
        top_bar.set_on_open_settings(move || {
            m.settings_status.set(String::new());
            m.settings_open.set(true);
        });
    }

    metrics_bar.set_on_restart({
        let c = Rc::clone(&core);
        move || c.borrow_mut().restart_or_exit()
    });
    metrics_bar.set_on_back_to_map({
        let c = Rc::clone(&core);
        move || c.borrow_mut().back_to_jungle_map()
    });

    jungle_panel.set_on_continue_next({
        let c = Rc::clone(&core);
        move || c.borrow_mut().start_next_jungle_lesson()
    });
    jungle_panel.set_on_select_stage({
        let c = Rc::clone(&core);
        move |idx| c.borrow_mut().select_stage(idx.max(0) as usize)
    });
    jungle_panel.set_on_select_unit({
        let c = Rc::clone(&core);
        move |idx| c.borrow_mut().select_visible_unit(idx.max(0) as usize)
    });
    jungle_panel.set_on_start_lesson({
        let c = Rc::clone(&core);
        move |id| c.borrow_mut().start_jungle_lesson(&id)
    });

    exam_bar.set_on_duration_index({
        let c = Rc::clone(&core);
        move |idx| c.borrow_mut().set_exam_duration_idx(idx.max(0) as usize)
    });
    exam_bar.set_on_prev_passage({
        let c = Rc::clone(&core);
        move || c.borrow_mut().step_exam_passage(-1)
    });
    exam_bar.set_on_next_passage({
        let c = Rc::clone(&core);
        move || c.borrow_mut().step_exam_passage(1)
    });
    exam_bar.set_on_toggle_strict({
        let c = Rc::clone(&core);
        move |strict| c.borrow_mut().set_exam_strict(strict)
    });
    {
        let eb = exam_bar.clone();
        exam_bar.set_on_toggle_custom_form(move || {
            let next = !eb.show_custom_form.get_untracked();
            eb.show_custom_form.set(next);
        });
    }
    exam_bar.set_on_save_custom({
        let c = Rc::clone(&core);
        move || c.borrow_mut().save_custom_passage()
    });
    exam_bar.set_on_reset_exam({
        let c = Rc::clone(&core);
        move || c.borrow_mut().start_exam_with_selected()
    });
    exam_bar.set_on_finish_exam({
        let c = Rc::clone(&core);
        move || c.borrow_mut().finish_exam()
    });

    analytics_panel.set_on_continue_jungle({
        let c = Rc::clone(&core);
        move || c.borrow_mut().select_tab(StudioTab::Jungle as usize)
    });
    analytics_panel.set_on_practice_weak_keys({
        let c = Rc::clone(&core);
        move || c.borrow_mut().start_weak_key_drill()
    });

    {
        let m = modals.clone();
        modals.set_on_dismiss_toast(move || m.show_unlock_toast.set(false));
    }
    modals.set_on_lesson_back_map({
        let c = Rc::clone(&core);
        move || c.borrow_mut().back_to_jungle_map()
    });
    modals.set_on_lesson_retry({
        let c = Rc::clone(&core);
        move || c.borrow_mut().restart_or_exit()
    });
    modals.set_on_lesson_next({
        let c = Rc::clone(&core);
        move || c.borrow_mut().start_next_jungle_lesson()
    });
    modals.set_on_exam_retry({
        let c = Rc::clone(&core);
        move || c.borrow_mut().start_exam_with_selected()
    });
    modals.set_on_exam_to_adaptive({
        let c = Rc::clone(&core);
        move || c.borrow_mut().select_tab(StudioTab::Adaptive as usize)
    });
    modals.set_on_settings_changed({
        let c = Rc::clone(&core);
        move || c.borrow_mut().apply_settings_from_modals()
    });
    modals.set_on_export_backup({
        let c = Rc::clone(&core);
        move || c.borrow().export_backup_from_modal()
    });
    modals.set_on_import_backup({
        let c = Rc::clone(&core);
        move || c.borrow_mut().import_backup_from_modal()
    });
    modals.set_on_reset_layout({
        let c = Rc::clone(&core);
        move || c.borrow_mut().reset_current_layout()
    });

    // Custom interactive widgets
    let stage_id = tree.custom_widget(
        cx,
        TypingStageWidget {
            core: Rc::clone(&core),
        },
        LayoutStyle::column().with_size(pct(100), dp(196)),
    );
    let keyboard_id = tree.custom_widget(
        cx,
        VirtualKeyboardWidget::new(Rc::clone(&core)),
        LayoutStyle::column().with_flex_grow(1.0),
    );
    let finger_id = tree.custom_widget(
        cx,
        FingerGuideWidget {
            core: Rc::clone(&core),
        },
        LayoutStyle::column().with_size(dp(240), dp(232)),
    );
    let balloon_id = tree.custom_widget(
        cx,
        BalloonMiniGameWidget::new(Rc::clone(&core)),
        LayoutStyle::column()
            .with_size(pct(100), dp(420))
            .with_flex_grow(1.0),
    );
    let chart_id = tree.custom_widget(
        cx,
        WpmTrendChartWidget {
            core: Rc::clone(&core),
        },
        LayoutStyle::column().with_size(pct(100), dp(170)),
    );

    // Repaint custom widgets whenever core state revision updates
    cx.effect(move || {
        let _ = revision.get();
        mark_node_dirty(stage_id, DirtyFlags::PAINT);
        mark_node_dirty(keyboard_id, DirtyFlags::PAINT);
        mark_node_dirty(finger_id, DirtyFlags::PAINT);
        mark_node_dirty(balloon_id, DirtyFlags::PAINT);
        mark_node_dirty(chart_id, DirtyFlags::PAINT);
    });

    // Bottom keyboard + finger guide row
    let kb_row = tree.row(cx);
    tree.set_layout(
        kb_row,
        LayoutStyle::row()
            .with_size(pct(100), auto())
            .with_gap(12.0),
    );
    tree.append_child(kb_row, keyboard_id);
    tree.append_child(kb_row, finger_id);

    // Typing workspace column (used for Adaptive, Exam, and Jungle typing lessons)
    let typing_col = tree.column(cx);
    tree.set_layout(
        typing_col,
        LayoutStyle::column()
            .with_size(pct(100), auto())
            .with_flex_grow(1.0)
            .with_gap(12.0),
    );
    tree.append_child(typing_col, exam_node);
    tree.append_child(typing_col, metrics_node);
    tree.append_child(typing_col, unlock_node);
    tree.append_child(typing_col, stage_id);
    tree.append_child(typing_col, kb_row);

    // Analytics wrapper (AnalyticsPanel + WpmTrendChartWidget)
    let analytics_wrap = tree.column(cx);
    tree.set_layout(
        analytics_wrap,
        LayoutStyle::column()
            .with_size(pct(100), auto())
            .with_flex_grow(1.0)
            .with_gap(12.0),
    );
    tree.append_child(analytics_wrap, analytics_node);
    tree.append_child(analytics_wrap, chart_id);

    // Visibility predicates keeping custom widget nodes permanently mounted
    tree.set_visible_when(jungle_node, move || {
        tab_sig.get() == StudioTab::Jungle as usize && !has_active_lesson_sig.get()
    });
    tree.set_visible_when(balloon_id, move || {
        tab_sig.get() == StudioTab::Jungle as usize
            && has_active_lesson_sig.get()
            && mode_sig.get() == PracticeMode::MiniGame
    });
    tree.set_visible_when(analytics_wrap, move || {
        tab_sig.get() == StudioTab::Analytics as usize
    });
    tree.set_visible_when(typing_col, move || {
        let t = tab_sig.get();
        t == StudioTab::Adaptive as usize
            || t == StudioTab::Exam as usize
            || (t == StudioTab::Jungle as usize
                && has_active_lesson_sig.get()
                && mode_sig.get() != PracticeMode::MiniGame)
    });
    tree.set_visible_when(exam_node, move || {
        !zen_sig.get() && tab_sig.get() == StudioTab::Exam as usize
    });
    tree.set_visible_when(metrics_node, move || !zen_sig.get());
    tree.set_visible_when(unlock_node, move || {
        !zen_sig.get() && tab_sig.get() == StudioTab::Adaptive as usize
    });
    tree.set_visible_when(kb_row, move || !zen_sig.get());

    // Main workspace padding frame
    let workspace = tree.column(cx);
    tree.set_layout(
        workspace,
        LayoutStyle::column()
            .with_size(pct(100), auto())
            .with_flex_grow(1.0)
            .with_min_size(dp(0), dp(0))
            .with_padding(Insets::symmetric(20.0, 14.0))
            .with_gap(12.0),
    );
    tree.append_child(workspace, jungle_node);
    tree.append_child(workspace, balloon_id);
    tree.append_child(workspace, analytics_wrap);
    tree.append_child(workspace, typing_col);

    let main_col = tree.column(cx);
    tree.set_layout(
        main_col,
        LayoutStyle::column().with_size(pct(100), pct(100)),
    );
    tree.set_box_style(
        main_col,
        BoxStyle {
            background: Some(ReactiveValue::Dynamic(Rc::new(move || {
                Brush::Solid(theme_sig.get().background)
            }))),
            ..Default::default()
        },
    );
    tree.append_child(main_col, top_bar_node);
    tree.append_child(main_col, workspace);

    let root_stack = tree.stack(cx);
    tree.set_layout(
        root_stack,
        LayoutStyle::column().with_size(pct(100), pct(100)),
    );
    tree.append_child(root_stack, main_col);
    tree.append_child(root_stack, modals_node);
    tree.set_root(root_stack);
    tree.set_focus(Some(stage_id), true);

    // 1-second timer for Exam Mode countdown
    let _exam_timer = cx.interval(1000, {
        let c = Rc::clone(&core);
        move || {
            if let Ok(mut guard) = c.try_borrow_mut() {
                guard.tick_exam_1s();
            }
        }
    });

    // ~60 FPS timer driving Balloon Pop Mini-Game when active
    let _game_timer = cx.interval(16, {
        let c = Rc::clone(&core);
        move || {
            if let Ok(mut guard) = c.try_borrow_mut() {
                guard.tick_game_frame(16.0);
            }
        }
    });

    (root_stack, core)
}
