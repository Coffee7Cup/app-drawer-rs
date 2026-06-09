use guido::{prelude::*, renderer::Gradient};

pub fn app_launcher() -> Container {
    let coord = create_signal((0.0, 0.0));

    let left_grad = container().gradient_horizontal(Color::WHITE, Color::TRANSPARENT);
    let right_grad = container().gradient_horizontal(Color::TRANSPARENT, Color::WHITE);

    container()
        .width(fill())
        .height(fill())
        .background(Color::rgba(255.0, 255.00, 255.0, 0.5))
        .border(2.0, Color::rgba(255.0, 255.0, 255.0, 0.7))
        .layout(
            Flex::row()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::Center),
        )
        .children([app_icon()])
}

pub fn app_icon() -> Container {
    let dist = create_signal((0.0, 0.0));

    container()
        .width(270)
        .height(90)
        .background(Color::TRANSPARENT)
        .animate_transform(Transition::spring(SpringConfig::GENTLE))
        .scale(move || {
            let (_, d) = dist.get();
            if d < 90.0 {
                ((181 * d) / 180) as f32
            } else if d >= 90.0 && d <= 180.0 {
                1.5 as f32
            } else {
                let abs = -1 * (270.0 - d);
                ((181 * d) / 180) as f32
            }
        })
        .on_pointer_move(move |x, y| {
            dist.set((x, y));
        })
        .children([image("./default-icon.png")])
}
