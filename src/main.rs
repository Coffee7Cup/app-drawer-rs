use std::{thread::sleep, time::Duration};

use guido::prelude::*;
mod app_launcher;
mod apps;
mod error;

fn main() {
    App::new().run(|app| {
        let id: RwSignal<Option<SurfaceId>> = create_signal(None);

        let launcher_id = app.add_surface(
            SurfaceConfig::new()
                .width(900)
                .height(70)
                .anchor(Anchor::BOTTOM)
                .layer(Layer::Overlay)
                .exclusive_zone(Some(0))
                .background_color(Color::rgb(0.1, 0.1, 0.15)),
            move || {
                container()
                    .height(fill())
                    .width(900.0)
                    .layout(
                        Flex::column()
                            .main_alignment(MainAlignment::Center)
                            .cross_alignment(CrossAlignment::Center),
                    )
                    .background(Color::BLACK)
                    .on_hover(move |hover| {
                        if let Some(lid) = id.get() {
                            let handle = surface_handle(lid);

                            if hover {
                                handle.set_size(900, 100);
                            } else {
                                handle.set_size(900, 70);
                            }
                        }
                    })
            },
        );

        id.set(Some(launcher_id));
    });
}

