use guido::prelude::*;
use std::time::Duration;
use tokio::time::sleep; // Use standard sleep for delay tasks cleanly

mod app_launcher;
mod apps;
mod error;

#[tokio::main]
async fn main() {
    App::new().run(|app| {
        let panel_id: RwSignal<Option<SurfaceId>> = create_signal(None);
        let panel_open: RwSignal<bool> = create_signal(false);

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
                        // Check state safely
                        let current_panel = panel_id.get_untracked();

                        if hover && current_panel.is_none() {
                            println!("Trigger hit: Launching panel...");

                            // 1. Instantly stage the structural transition signal states
                            panel_open.set(true);

                            let new_handle = spawn_surface(
                                SurfaceConfig::new()
                                    .width(900)
                                    .height(100)
                                    .anchor(Anchor::BOTTOM)
                                    .layer(Layer::Overlay)
                                    .exclusive_zone(Some(0)),
                                move || {
                                    // KEEP CLOSURE CLEAN: Avoid running naked async timeouts directly inside the view tree return
                                    container()
                                        .height(fill())
                                        .width(fill())
                                        .background(Color::from_rgba8(255, 255, 255, 50))
                                        .on_hover(move |inside_panel| {
                                            if !inside_panel {
                                                println!(
                                                    "Mouse left panel: Triggering close animation."
                                                );

                                                // Trigger slide-down transition state immediately
                                                panel_open.set(false);

                                                // Extract what we need safely right now *before* entering async context
                                                let target_id = panel_id.get_untracked();

                                                // Spawn async side effect cleanly without dragging ownership into layout evaluations
                                                tokio::spawn(async move {
                                                    // Give the 300ms EaseOut animation time to finish sliding down
                                                    sleep(Duration::from_millis(300)).await;

                                                    if let Some(id) = target_id {
                                                        surface_handle(id).close();
                                                    }
                                                    // Reset state trackers
                                                    panel_id.set(None);
                                                });
                                            }
                                        })
                                        .animate_transform(Transition::new(
                                            300,
                                            TimingFunction::EaseOut,
                                        ))
                                        .transform(move || {
                                            if panel_open.get() {
                                                Transform::identity()
                                            } else {
                                                Transform::translate(0.0, 100.0)
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

