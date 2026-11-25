pub mod app;
pub mod fonts;
pub mod state;
pub mod widgets;

pub use app::mount_aksharflow_app;
pub use state::{AppCore, StudioTab};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "aksharflow_backup_v1";

#[cfg(target_arch = "wasm32")]
fn web_local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start_web() -> Result<(), JsValue> {
    use aksharflow_core::session::SessionEngine;
    use aksharflow_core::storage::MemoryStorage;
    use craftui_platform_web::{WebBridgeConfig, start_web_app};

    start_web_app(
        WebBridgeConfig {
            canvas_id: "craftui-canvas".to_string(),
            sync_url_hash: true,
            ..Default::default()
        },
        move |cx, tree, _textures, _dispatcher| {
            let storage = web_local_storage()
                .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
                .and_then(|raw| MemoryStorage::from_backup_json(&raw).ok())
                .unwrap_or_else(MemoryStorage::new);

            let engine = SessionEngine::with_storage(storage)
                .expect("failed initializing AksharFlow core engine");
            let (root, core) = mount_aksharflow_app(cx, tree, engine);

            core.borrow_mut().persist_hook = Some(Rc::new(|json| {
                if let Some(st) = web_local_storage() {
                    let _ = st.set_item(STORAGE_KEY, json);
                }
            }));

            core.borrow_mut().export_hook = Some(Rc::new(|json| {
                if let Some(st) = web_local_storage() {
                    let _ = st.set_item(STORAGE_KEY, json);
                }
                Ok("browser localStorage + download".to_string())
            }));

            root
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use aksharflow_core::keymap::LayoutKind;
    use aksharflow_core::session::{PracticeMode, SessionEngine};
    use aksharflow_core::storage::MemoryStorage;
    use craftui::prelude::*;
    use craftui::reactive::reset_runtime;

    #[test]
    fn craftui_mount_renders_devanagari_and_drives_all_four_modes() {
        reset_runtime();
        let cx = Scope::root();
        let mut tree = WidgetTree::new(cx);
        let textures = TextureRegistry::new();
        let engine = SessionEngine::with_storage(MemoryStorage::new()).unwrap();
        let (root, core) = mount_aksharflow_app(cx, &mut tree, engine);
        tree.set_root(root);

        let metrics = WindowMetrics {
            logical_size: Size::new(1240.0, 820.0),
            ..Default::default()
        };
        tree.layout(metrics);

        // Switch to Hindi Remington GAIL layout & type with CapsLock ON (safeguard ignores CapsLock)
        core.borrow_mut().select_layout(2);
        tree.layout(metrics);
        assert_eq!(core.borrow().layout, LayoutKind::HiRemingtonGail);
        let first_expected_code = core.borrow().next_key_hint.as_ref().unwrap().code.clone();
        let kc = KeyCode::from_dom_code(&first_expected_code).unwrap();
        assert!(tree.handle_event(&UiEvent::Key(KeyEvent {
            code: kc,
            text: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                caps_lock: true,
                ..Default::default()
            },
        })));
        assert!(core.borrow().metrics.keystrokes >= 1);

        // Switch to Jungle Journey, start Unit 1 Lesson 10 (Balloon Pop Mini-Game), tick frames
        core.borrow_mut().select_tab(StudioTab::Jungle as usize);
        tree.layout(metrics);
        assert!(core.borrow().roadmap.as_ref().unwrap().total_lessons == 700);

        core.borrow_mut().start_jungle_lesson("u1-l10");
        tree.layout(metrics);
        assert_eq!(core.borrow().mode, PracticeMode::MiniGame);
        core.borrow_mut().tick_game_frame(2200.0);
        assert!(!core.borrow().balloon.balloons.is_empty());

        // Switch to Exam Mode & Analytics, then CPU-render full scene cleanly
        core.borrow_mut().select_tab(StudioTab::Exam as usize);
        tree.layout(metrics);
        assert_eq!(core.borrow().mode, PracticeMode::Exam);

        core.borrow_mut().select_tab(StudioTab::Analytics as usize);
        tree.layout(metrics);

        let mut scene = PaintScene::new();
        tree.paint(&mut scene);
        let rgba = render_scene_to_rgba(
            BackendKind::VelloCpu,
            &scene,
            1240,
            820,
            1.0,
            tree.theme().background,
            &textures,
            &mut tree.text_engine,
        );
        assert_eq!(rgba.len(), 1240 * 820 * 4);
        assert!(rgba.iter().any(|&b| b > 200));
    }

    #[test]
    fn autofocus_tab_enter_and_virtual_keyboard_click_drive_session() {
        reset_runtime();
        let cx = Scope::root();
        let mut tree = WidgetTree::new(cx);
        let engine = SessionEngine::with_storage(MemoryStorage::new()).unwrap();
        let (root, core) = mount_aksharflow_app(cx, &mut tree, engine);
        tree.set_root(root);

        let metrics = WindowMetrics {
            logical_size: Size::new(1240.0, 820.0),
            ..Default::default()
        };
        tree.layout(metrics);

        // Navigate to Jungle Map (which hides TypingStageWidget), then open Unit 1 Lesson 1.
        core.borrow_mut().select_tab(StudioTab::Jungle as usize);
        tree.layout(metrics);
        core.borrow_mut().start_jungle_lesson("u1-l1");
        tree.layout(metrics);
        assert_eq!(core.borrow().mode, PracticeMode::Jungle);

        // First keystroke immediately works via autofocus() without needing a canvas click.
        let first_code = core.borrow().next_key_hint.as_ref().unwrap().code.clone();
        let kc = KeyCode::from_dom_code(&first_code).unwrap();
        assert!(tree.handle_event(&UiEvent::Key(KeyEvent {
            code: kc,
            text: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        })));
        assert!(core.borrow().metrics.keystrokes >= 1);

        // Tab followed by Enter restarts the lesson cleanly.
        assert!(tree.handle_event(&UiEvent::Key(KeyEvent {
            code: KeyCode::Tab,
            text: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        })));
        assert!(tree.handle_event(&UiEvent::Key(KeyEvent {
            code: KeyCode::Enter,
            text: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        })));
        assert_eq!(core.borrow().metrics.keystrokes, 0);
        assert_eq!(core.borrow().grapheme_index, 0);

        // Tapping a virtual keyboard keycap dispatches cleanly into the active session.
        let next_code = core.borrow().next_key_hint.as_ref().unwrap().code.clone();
        core.borrow_mut().press_virtual_key(&next_code);
        assert!(core.borrow().metrics.keystrokes >= 1);
    }

    #[test]
    fn jungle_lesson_completion_opens_modal_and_persists_backup_json() {
        use std::cell::RefCell;
        use std::rc::Rc;

        reset_runtime();
        let cx = Scope::root();
        let mut tree = WidgetTree::new(cx);
        let engine = SessionEngine::with_storage(MemoryStorage::new()).unwrap();
        let (root, core) = mount_aksharflow_app(cx, &mut tree, engine);
        tree.set_root(root);
        tree.layout(WindowMetrics::default());

        let persisted = Rc::new(RefCell::new(None::<String>));
        let slot = Rc::clone(&persisted);
        core.borrow_mut().persist_hook = Some(Rc::new(move |json| {
            *slot.borrow_mut() = Some(json.to_string());
        }));

        core.borrow_mut().select_layout(0);
        core.borrow_mut().start_jungle_lesson("u1-l1");
        core.borrow_mut()
            .complete_active_jungle_lesson(Some((38.0, 96.0, 1, 50)));

        let c = core.borrow();
        assert!(c.modals.lesson_modal_open.get());
        let done = c.lesson_completion.as_ref().unwrap();
        assert!(done.stars_earned >= 3);
        let backup_json = persisted.borrow().clone().expect("persist_hook must fire");
        assert!(backup_json.contains("u01-l01"));
        drop(c);

        // Verify the persisted JSON round-trips cleanly back into MemoryStorage.
        let restored = MemoryStorage::from_backup_json(&backup_json).unwrap();
        let restored_engine = SessionEngine::with_storage(restored).unwrap();
        let roadmap = restored_engine
            .get_jungle_roadmap(LayoutKind::EnQwerty)
            .unwrap();
        let first_lesson = &roadmap.units[0].lessons[0];
        assert_eq!(first_lesson.id, "u01-l01");
        assert!(first_lesson.stars >= 3);
    }
}
