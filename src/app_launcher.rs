use guido::prelude::*;

pub fn invisible_line() -> Container {
    let height = create_signal(100);

    container()
        .height(move || height.get())
        .width(fill())
        .background(Color::BLACK)
        .animate_height(Transition::new(300, TimingFunction::EaseOut))
        .on_hover(move |hover| {
            if hover {
                height.set(0);
                println!("Hoverd in")
            } else {
                height.set(100);
                println!("Hover out")
            }
        })
}

pub fn main_content() -> Container {
    container()
        .height(fill())
        .width(fill())
        .background(Color::CYAN)
}
