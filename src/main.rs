mod app_launcher;
mod apps;
mod config;
mod current_apps;
mod error;
mod init_app;
mod workspaces;

use std::rc::Rc;

use guido::prelude::*;

use crate::config::Congif;

fn default_surface_config() -> SurfaceConfig {
    SurfaceConfig::new()
        .layer(Layer::Overlay)
        .background_color(Color::TRANSPARENT)
        .exclusive_zone(Some(0))
}

fn main() {
    App::new().run(|app| {
        // ------------------------------------- set up ---------------------------------//
        let niri_apps = match apps::NiriApp::get_apps() {
            Ok(val) => val,
            Err(e) => {
                let _ = notify_rust::Notification::new()
                    .summary("Launcher Error")
                    .body(&format!("Failed to scan apps: {}", e))
                    .urgency(notify_rust::Urgency::Critical)
                    .show();
                std::process::exit(1);
            }
        };

        let config_raw = match init_app::app_config() {
            Ok(val) => val,
            Err(e) => {
                let _ = notify_rust::Notification::new()
                    .summary("Launcher Config Error")
                    .body(&format!("Failed to parse config: {}", e))
                    .urgency(notify_rust::Urgency::Critical)
                    .show();
                std::process::exit(1);
            }
        };

        let config = Rc::new(config_raw);
        let cache_file = match init_app::AccessCache::read_cache_file(config.clone()) {
            Ok(val) => val,
            Err(e) => {
                let _ = notify_rust::Notification::new()
                    .summary("Launcher Error")
                    .body(&format!("Failed to scan cache_file: {}", e))
                    .urgency(notify_rust::Urgency::Critical)
                    .show();
                std::process::exit(1);
            }
        };

        // -------------------------------- for context ----------------------------------------------

        provide_context(config.clone());
        provide_signal_context(cache_file); // im providing the cache file as context since i have me make 4 passes to apps.rs::open() to use it

        //------------------------------ app launcher --------------------------------------------------------//
        let conf_h = config.clone();
        let h0 = create_signal(false);
        let niri_apps_h = niri_apps.clone();

        let id = app.add_surface(
            default_surface_config()
                .width(conf_h.window_width)
                .height(conf_h.trigger_area)
                .anchor(Anchor::BOTTOM),
            move || {
                container()
                    .width(conf_h.window_width)
                    .height(conf_h.window_height)
                    .background(Color::TRANSPARENT)
                    .children([
                        container()
                            .height(conf_h.trigger_area)
                            .width(conf_h.window_width)
                            .on_hover(move |h| {
                                if h {
                                    h0.set(h)
                                }
                            })
                            .background(Color::TRANSPARENT),
                        container()
                            .width(conf_h.window_width)
                            .height(conf_h.window_height - conf_h.trigger_area)
                            .on_hover(move |h| {
                                if !h {
                                    h0.set(h)
                                }
                            })
                            .background(Color::TRANSPARENT)
                            .children([
                                container()
                                    .width(conf_h.window_width)
                                    .height(move || {
                                        if h0.get() {
                                            0
                                        } else {
                                            conf_h.window_height - conf_h.trigger_area
                                        }
                                    })
                                    .animate_height(Transition::new(100, TimingFunction::EaseOut))
                                    .background(Color::TRANSPARENT),
                                app_launcher::app_launcher(niri_apps_h),
                            ]),
                    ])
            },
        );

        //---------------------------------- effects -------------------------------//
        let conf2 = config.clone();
        create_effect(move || {
            let hovering = h0.get();

            if hovering {
                surface_handle(id).set_size(conf2.window_width, conf2.window_height);
            } else {
                surface_handle(id).set_size(conf2.window_width, conf2.trigger_area);
            }
        });

        //######################################################### currect app switcher ##############################################################

        let conf_v = config.clone();
        let conf_v_t = config.clone();
        let h0_v = create_signal(false);
        let niri_apps_v = niri_apps.clone();

        let v_id = app.add_surface(
            default_surface_config()
                .height(conf_v.window_length)
                .width(conf_v.trigger_area)
                .anchor(Anchor::LEFT),
            move || {
                container()
                    .width(move || {
                        if h0_v.get() {
                            conf_v_t.window_thickness
                        } else {
                            conf_v_t.trigger_area
                        }
                    })
                    .on_hover(move |h| {
                        if h {
                            h0_v.set(h)
                        };
                    })
                    .height(conf_v.window_length)
                    .layout(
                        Flex::row()
                            .main_alignment(MainAlignment::Center)
                            .cross_alignment(CrossAlignment::Center),
                    )
                    .children([container()
                        .height(conf_v.window_length)
                        .layout(
                            Flex::row()
                                .main_alignment(MainAlignment::Center)
                                .cross_alignment(CrossAlignment::Center),
                        )
                        .width((conf_v.window_thickness - conf_v.trigger_area) * 2)
                        .on_hover(move |h| {
                            if !h {
                                h0_v.set(h);
                            }
                        })
                        .children([
                            current_apps::current_apps(niri_apps_v.to_vec()),
                            container()
                                .height(conf_v.window_height)
                                .width(move || {
                                    if h0_v.get() {
                                        conf_v.window_thickness - conf_v.trigger_area
                                    } else {
                                        0
                                    }
                                })
                                .animate_width(Transition::new(100, TimingFunction::EaseOut)),
                        ])])
            },
        );

        let conf_v2 = config.clone();

        create_effect(move || {
            let hovering = h0_v.get();

            if hovering {
                surface_handle(v_id).set_size(conf_v2.window_thickness, conf_v2.window_length);
            } else {
                surface_handle(v_id).set_size(conf_v2.trigger_area, conf_v2.window_length);
            }
        });
    });
}
