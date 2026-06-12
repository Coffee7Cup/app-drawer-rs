use std::time::Duration;

use guido::prelude::*;
use tokio::time::{sleep, timeout};
mod app_launcher;
mod apps;
mod error;

#[tokio::main]
async fn main() {
    App::new().run(|app| {
        let panel_id: RwSignal<Option<SurfaceId>> = create_signal(None);
        let is_plane_visible = create_signal(false);
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
                                    is_plane_visible.set(true);

                                    container()
                                        .height(fill())
                                        .width(fill())
                                        .background(Color::rgba(255.0, 255.0, 255.0, 0.5))
                                        .transform(Transform::translate(0.0, -100.0))
                                        .on_hover(move |inside_panel| {
                                            if !inside_panel {
                                                println!("Mouse left panel: Destroying.");
                                                is_plane_visible.set(false);
                                                let id = panel_id.get_untracked();

                                                tokio::spawn(async move {
                                                    sleep(Duration::from_millis(200)).await;
                                                    if let Some(id) = id {
                                                        surface_handle(id).close();
                                                    }
                                                });

                                                panel_id.set(None);
                                            }
                                        })
                                        // WARN:  the children are causing "i guess" dangling pointer error Signal 69 was disposed - cannot read after owner cleanup. This usually means the signal's owner was disposed while you still hold a reference to the signal.
                                        .children([
                                            container()
                                                .width(fill())
                                                .height(move || {
                                                    if is_plane_visible.get() { 0 } else { 100 }
                                                })
                                                .background(Color::rgba(255.0, 0.0, 0.0, 1.0))
                                                .animate_height(Transition::new(
                                                    200,
                                                    TimingFunction::EaseInOut,
                                                )),
                                            app_launcher::app_launcher(),
                                        ])
                                },
                            );
                            if panel_id.get().is_none() {
                                panel_id.set(Some(new_handle.id()));
                            }
                        }
                    })
            },
        );
    });
}

