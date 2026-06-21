use std::{rc::Rc, sync::Arc};

use guido::prelude::*;

use crate::{
    app_launcher::app_icon,
    apps::NiriApp,
    config::Congif,
    workspaces::{self, focus_window},
};

#[derive(Clone, Debug)]
pub struct ActiveAppEntry {
    pub app: Arc<NiriApp>,
    pub window: niri_ipc::Window,
}

impl PartialEq for ActiveAppEntry {
    fn eq(&self, other: &Self) -> bool {
        self.window.id == other.window.id
    }
}

pub fn current_apps(niri_apps: Vec<Arc<NiriApp>>) -> Container {
    let hover = create_signal(false);
    let conf = expect_context::<Rc<Congif>>();

    // 💡 1. Store clonable data structs in the signal
    let active_apps: RwSignal<Vec<ActiveAppEntry>> = create_signal(Vec::new());

    create_effect(move || {
        if hover.get() {
            let current_windows = match workspaces::get_windows() {
                Ok(val) => val,
                Err(_) => {
                    let _ = notify_rust::Notification::new()
                        .summary("app-launcher")
                        .body("Error in current_apps")
                        .show();
                    std::process::exit(1);
                }
            };

            let mut matched_entries: Vec<ActiveAppEntry> = Vec::new();

            for capp in current_windows {
                if let Some(ref app_id) = capp.app_id {
                    if let Some(napp) = niri_apps.iter().find(|f| f.icon.as_ref() == Some(app_id)) {
                        matched_entries.push(ActiveAppEntry {
                            app: napp.clone(),
                            window: capp,
                        });
                    }
                }
            }
            active_apps.set(matched_entries);
        }
    });

    //TODO: add the caption to the icons
    container()
        .width(conf.window_thickness)
        .height(fill())
        .scrollable(ScrollAxis::Vertical)
        .scrollbar(|sb| sb.width(0.0))
        .on_hover(move |h| {
            hover.set(h);
        })
        .layout(
            Flex::column()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::Center),
        )
        .background(Color::rgba(0.0, 0.0, 0.0, 0.5))
        .border(2.0, Color::rgba(255.0, 255.0, 255.0, 0.7))
        .corner_radius(20.0)
        .squircle()
        .padding([10, 3])
        .children(move || {
            // from docs: Accepts Fn() -> Iterator<Item = (key, FnOnce() -> Widget)>.
            active_apps
                .get()
                .iter()
                .map(|app| {
                    let win_id = app.window.id;
                    let win_clone = app.window.clone();
                    let app_arc = app.app.clone();

                    (win_id, move || {
                        app_icon(app_arc, move || focus_window(&win_clone))
                    })
                })
                .collect::<Vec<_>>()
        })
}
