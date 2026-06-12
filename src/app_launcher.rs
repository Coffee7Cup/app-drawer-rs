use guido::prelude::*;
//i want the size of each app to be 90x90
//and want to animate the size when the cursor is just with in the adjecent app.
const ICON_DIM: f32 = 90f32;
const GAP: f32 = 5f32;
pub fn app_launcher() -> Container {
    container()
        .width(fill())
        .height(fill())
        .background(Color::rgba(255.0, 255.0, 255.0, 0.5))
        .border(2.0, Color::rgba(255.0, 255.0, 255.0, 0.7))
        .padding([3, -90])
        .layout(
            Flex::row()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::End)
                .spacing(-(2.0 * ICON_DIM)), // Adjusted spacing so scaled icons don't clip drastically
        )
        .scrollable(ScrollAxis::Horizontal)
        .children([app_icon(), app_icon(), app_icon(), app_icon(), app_icon()])
}
pub fn app_icon() -> Container {
    // Track the pointer's distance from the center of this icon.
    // Default to a large number so it starts at base scale (1.0).
    let distance: RwSignal<Option<f32>> = create_signal(None);
    container()
        .width(280) // Reduced container width to match the base icon size
        .height(90)
        .background(Color::TRANSPARENT)
        .layout(
            Flex::row()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::End),
        )
        .animate_transform(Transition::new(4, TimingFunction::EaseOut))
        .scale(move || {
            let pad = GAP + ICON_DIM;
            let max = 1.3f32;
            let min = 1.0f32;
            if let Some(x) = distance.get() {
                if x > 0f32 && x < 95f32 {
                    (((max - min) / pad) * x) + min
                } else if (95f32..=185f32).contains(&x) {
                    max
                } else {
                    let _x = (GAP * 2.0 + ICON_DIM * 3.0) - x;
                    (((max - min) / pad) * _x) + min
                }
            } else {
                1.0
            }
        })
        .on_pointer_move(move |x, _| distance.set(Some(x)))
        // Reset scale when the pointer completely leaves the icon boundaries
        .on_hover(move |hovering| {
            if !hovering {
                distance.set(None);
            }
        })
        .squircle()
        .children([image("./default-icon.png")
            .width(90)
            .height(90)
            .content_fit(ContentFit::Contain)])
}
