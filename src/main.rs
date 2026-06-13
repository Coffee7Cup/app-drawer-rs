use std::time::Duration;

use guido::prelude::*;
use tokio::time::{sleep, timeout};
mod app_launcher;
mod apps;
mod error;
mod init_app;

#[tokio::main]
async fn main() {
    App::new().run(|app| {
        provide_context(apps::NiriApp::get_apps());
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
                                    container()
                                        .height(fill())
                                        .width(fill())
                                        .background(Color::TRANSPARENT)
                                        .on_hover(move |inside_panel| {
                                            if !inside_panel {
                                                println!("Mouse left panel: Destroying.");
                                                let id = panel_id.get_untracked();

                                                if let Some(id) = id {
                                                    surface_handle(id).close();
                                                }

                                                panel_id.set(None);
                                            }
                                        })
                                        //not happen wehn the cursor goes from below
                                        //WARN:  the children are causing "i guess" dangling pointer error Signal 69 was disposed - cannot read after owner cleanup. This usually means the signal's owner was disposed while you still hold a reference to the signal.
                                        .children([app_launcher::app_launcher()])
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
