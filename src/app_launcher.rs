use guido::{prelude::*, renderer::Gradient};

pub fn app_launcher() -> Container {
    container()
        .width(900)
        .height(100)
        .background(Color::rgba(255.0, 255.0, 255.0, 0.5))
        .border(2.0, Color::rgba(255.0, 255.0, 255.0, 0.7))
        .padding(3)
        .layout(
            Flex::row()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::Center)
                .spacing(-180), // Adjusted spacing so scaled icons don't clip drastically
        )
        .scrollable(ScrollAxis::Horizontal)
        .children([app_icon(), app_icon(), app_icon(), app_icon(), app_icon()])
}

pub fn app_icon() -> Container {
    // Track the pointer's distance from the center of this icon.
    // Default to a large number so it starts at base scale (1.0).
    let distance = create_signal(999.0);

    container()
        .width(280) // Reduced container width to match the base icon size
        .height(90)
        .background(Color::TRANSPARENT)
        .layout(
            Flex::row()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::Center),
        )
        .animate_transform(Transition::spring(SpringConfig::GENTLE))
        .scale(move || {
            let d = distance.get();
            let max_effect_radius = 150.0; // Distance where magnification begins
            let max_scale = 1.5; // Peak scale factor when cursor is exactly on center

            if d < max_effect_radius {
                // Smooth cosine interpolation for a natural "dock wave" effect
                let progress = (d / max_effect_radius) * std::f32::consts::FRAC_PI_2;
                let factor = progress.cos(); // 1.0 at center, 0.0 at boundary
                1.0 + (max_scale - 1.0) * factor
            } else {
                1.0 // Base scale
            }
        })
        .on_pointer_move(move |x, y| {
            // Assuming (x, y) are local coordinates relative to top-left of a 90x90 box:
            let center_x = 45.0;
            let center_y = 45.0;

            let dx = x - center_x;
            let dy = y - center_y;

            // Euclidean distance formula
            let dist = (dx * dx + dy * dy).sqrt();
            distance.set(dist);
        })
        // Reset scale when the pointer completely leaves the icon boundaries
        .on_hover(move |hovering| {
            if !hovering {
                distance.set(999.0);
            }
        })
        .squircle()
        .children([image("./default-icon.png")
            .width(90)
            .height(90)
            .content_fit(ContentFit::Contain)])
}
