use eframe::egui;
use egui::{Vec2, Color32, Sense, Pos2, Rect};
use fastrand;


#[derive(Default, Clone, Copy)]
struct LaserPointer {
    x: f32,
    y: f32,
    speed: Speed,
    imaginary_target: Pos2,
}

#[derive(Clone, Copy, Default)]
enum Speed {
    #[default]
    Still,
    Slow,
    Fast,
    CrazyFast,
}

//Implementing From here isn’t essential, but it helps the following code be a bit cleaner.
impl From<LaserPointer> for Pos2 {
        fn from(pointer: LaserPointer) -> Self {
            Pos2 {
                x: pointer.x,
                y: pointer.y,
            }
        }
}

impl LaserPointer {
    //This method now handles the random laser pointer movement when the mouse arrow gets too close.
    fn random_movement(&mut self, amount: f32) {
        if fastrand::bool() {
            self.x += fastrand::f32() * amount;
            } else {
            self.x -= fastrand::f32() * amount;
            }
            if fastrand::bool() {
                self.y += fastrand::f32() * amount;
            } else {
                self.y -= fastrand::f32() * amount;
            }
    }
    //We don’t want the speed to change too frequently (cats get bored when a laser pointer moves too quickly), so
    //we’ll use a random f32 from 0.0 to 1.0 and only change when the number is greater than 0.98. In practice, this
    //will mean a speed change every few seconds. The following try_change_target() changes the invisible target
    //for the pointer in the same way.
    fn try_change_speed(&mut self) {
            use Speed::*;
            if fastrand::f32() > 0.98 {
                self.speed = match fastrand::u8(0..3) {
                    0 => Still,
                    1 => Slow,
                    2 => Fast,
                    _ => CrazyFast,
            }
        }
    }
    fn try_change_target(&mut self, rect: Rect) {
        let bottom_right = rect.max;
        if fastrand::f32() > 0.98 {
            self.imaginary_target = Pos2 {
            x: fastrand::f32() * bottom_right.x,
            y: fastrand::f32() * bottom_right.y,
            }
        }
    }
    fn change_speed(&self) -> f32 {
        match self.speed {
            Speed::Still => 0.0,
            Speed::Slow => 0.05,
            Speed::Fast => 0.1,
            Speed::CrazyFast => 0.3,
        }
    }
    //Finally, we have this method to move the laser pointer once every loop. One of the speeds is 0.0,
    //though, so it will stay absolutely still in that case.
    fn move_self(&mut self) {
        let x_from_target = self.imaginary_target.x - self.x;
        let y_from_target = self.imaginary_target.y - self.y;
        self.x += fastrand::f32() * x_from_target * self.change_speed();
        self.y += fastrand::f32() * y_from_target * self.change_speed();
    }
}

impl LaserPointer {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
        x: 50.0,
        y: 50.0,
        speed: Speed::default(),
        imaginary_target: Pos2 { x: 50.0, y: 50.0 },
        }
    }
}

impl eframe::App for LaserPointer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ctx.screen_rect();
            self.try_change_speed();
            self.try_change_target(rect);
            self.move_self();
            
            let screen_size = Vec2 {
                x: rect.width(),
                y: rect.height()
            };
            let (_, painter) = ui.allocate_painter(screen_size,Sense::hover());
            let LaserPointer { x, y, .. } = self;
            let Pos2 { x: x2, y: y2 } =ctx.pointer_hover_pos().unwrap_or_default();
            if (*x - x2).abs() < 20.0 && (*y - y2).abs() < 20.0 {
                self.random_movement(50.0);
            }
            painter.circle_filled(Pos2::from(*self), 20.0, Color32::RED);
            //Display the invisible target position
            painter.circle_filled(self.imaginary_target, 10.0, Color32::GREEN);
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
    "Awesome laser pointer",
    native_options,
    Box::new(|cc| Ok(Box::new(LaserPointer::new(cc)))),
    );
}