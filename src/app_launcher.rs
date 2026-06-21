use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::Congif;
use crate::apps::NiriApp;
use crate::init_app::AccessCache;
use guido::prelude::*;

pub fn app_launcher(apps: Arc<[Arc<NiriApp>]>) -> Container {
    let access_map = expect_context::<RwSignal<AccessCache>>();
    let conf = expect_context::<Rc<Congif>>();

    let mut sorted_apps = apps.to_vec();
    let mut dead_apps = Vec::new();

    let cache_snapshot = access_map.get();

    sorted_apps.sort_by_key(|app| {
        let score = cache_snapshot
            .cache
            .iter()
            .find(|(name, _)| name == &app.name)
            .map(|(_, value)| *value)
            .unwrap_or(0);

        std::cmp::Reverse(score)
    });

    for (cached_name, _) in &cache_snapshot.cache {
        if !sorted_apps.iter().any(|app| &app.name == cached_name) {
            dead_apps.push(cached_name.clone());
        }
    }

    if !dead_apps.is_empty() {
        access_map.update(|cache| {
            for dead_app in dead_apps {
                cache.del_from_cache(&dead_app);
            }
        });
    }

    let app_icons: Vec<Container> = sorted_apps
        .iter()
        .map(|app| {
            let app_clone = app.clone();
            app_icon(app.clone(), move || {
                let _ = app_clone.open();
            })
        })
        .collect();

    let offset = create_signal(0.0f32);

    container()
        .width(fill())
        .height(fill())
        .background(Color::rgba(0.0, 0.0, 0.0, 0.5))
        .border(2.0, Color::rgba(255.0, 255.0, 255.0, 0.7))
        .corner_radius(20.0)
        .squircle()
        .padding(3)
        .on_scroll(move |dx, dy, _| {
            if dy != 0.0 {
                offset.update(|o| *o += dy);
            } else if dx != 0.0 {
                offset.update(|o| *o += dx);
            }
        })
        .children([container()
            .width(fill())
            .height(fill())
            .layout(
                Flex::row()
                    .main_alignment(MainAlignment::Center)
                    .cross_alignment(CrossAlignment::Center)
                    .spacing(conf.icon_gap),
            )
            .scrollable(ScrollAxis::Horizontal)
            .scrollbar(|f| f.width(0.0))
            .padding(move || {
                if offset.get() > 0.0 {
                    Padding::all(0.0).left(-offset.get())
                } else {
                    Padding::all(0.0)
                }
            })
            .children(app_icons)])
}

// now i can reuse this app_icon in current_apps
pub fn app_icon<F>(app: Arc<NiriApp>, callback: F) -> Container
where
    F: Fn() + 'static,
{
    let hovering: RwSignal<bool> = create_signal(false);
    let conf = expect_context::<Rc<Congif>>();

    let dim = conf.icon_dim;
    let factor = 40f32;

    let img_src = match (&app.icon_path, &app.icon) {
        (Some(path_buf), _) => {
            let path_str = path_buf.to_string_lossy();
            if path_str.ends_with(".svg") {
                ImageSource::SvgPath(path_str.into_owned().into())
            } else {
                ImageSource::Path(path_str.into_owned().into())
            }
        }
        (None, Some(icon_val)) => {
            let path = PathBuf::from(icon_val);
            if path.is_file() {
                ImageSource::Path(path)
            } else {
                ImageSource::Bytes(include_bytes!("assets/default-icon.png").as_slice().into())
            }
        }
        (None, None) => {
            ImageSource::Bytes(include_bytes!("assets/default-icon.png").as_slice().into())
        }
    };

    container()
        .width(dim)
        .height(dim)
        .background(Color::TRANSPARENT)
        .layout(
            Flex::column()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::Center),
        )
        .animate_transform(Transition::new(100, TimingFunction::EaseOut))
        .scale(move || if hovering.get() { 1.3 } else { 1.0 })
        .on_hover(move |h| hovering.set(h))
        .children([container()
            .width(dim - factor)
            .height(dim - factor)
            .on_click(move || {
                callback();
            })
            .child(
                image(img_src)
                    .width(50)
                    .height(50)
                    .content_fit(ContentFit::Contain),
            )])
}
