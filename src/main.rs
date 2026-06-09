use std::{thread::sleep, time::Duration};

use guido::prelude::*;
mod app_launcher;
mod apps;
mod error;

fn main() {
    App::new().run(|app| {
        // Track the ID of the spawned panel. None = hidden, Some(id) = visible.
        let panel_id: RwSignal<Option<SurfaceId>> = create_signal(None);
        let _launcher_id = app.add_surface(
            SurfaceConfig::new()
                .width(900)
                .height(5)
                .anchor(Anchor::BOTTOM)
                .layer(Layer::Overlay)
                .exclusive_zone(Some(0)),
            move || {
                container()
                    .height(fill())
                    .width(900.0)
                    .background(Color::TRANSPARENT)
                    .on_hover(move |hover| {
                        if hover && panel_id.get().is_none() {
                            println!("Trigger hit: Launching panel...");
                            let new_handle = spawn_surface(
                                SurfaceConfig::new()
                                    .width(900)
                                    .height(100)
                                    .anchor(Anchor::BOTTOM)
                                    .layer(Layer::Overlay)
                                    .exclusive_zone(Some(0)),
                                move || {
                                    // 2. THE LAUNCHER PANEL PANEL
                                    container()
                                        .height(fill())
                                        .width(fill())
                                        .background(Color::from_rgba8(255, 255, 255, 50))
                                        .on_hover(move |inside_panel| {
                                            if !inside_panel {
                                                println!("Mouse left panel: Destroying.");
                                                if let Some(id) = panel_id.get() {
                                                    surface_handle(id).close();
                                                }
                                                panel_id.set(None);
                                            }
                                        })
                                        .children([app_launcher::app_launcher()])
                                },
                            );
                            panel_id.set(Some(new_handle.id()));
                        }
                    })
            },
        );
    });
}
