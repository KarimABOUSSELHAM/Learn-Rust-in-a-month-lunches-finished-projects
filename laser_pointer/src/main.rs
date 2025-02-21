use eframe::egui;
use egui::{Vec2, Color32, Sense, Pos2, Rect};
use fastrand;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct LaserPointer {
    x: f32,
    y: f32,
    speed: Speed,
    imaginary_target: Pos2,
    screen_state: ScreenState,//This is to handle the screen mode after each button is clicked
    cursor_tracker: CursorTracker,//This stores the last duration and start time where the mouse has not moved
    // and handles the cursor movement for the restless cat
}

#[derive(PartialEq, Clone, Copy, Default)]
enum Speed {
    #[default]
    Still,
    Slow,
    Fast,
    CrazyFast,
}

// Add the following enum to distinguish among menu state and other states of the application 
#[derive(PartialEq, Clone, Copy, Default)]
enum ScreenState {
    #[default]
    Menu,
    Algorithm(usize),// This defines the algorithm for each of the three modes: 1 matches the restless cat, 2 matches the normal one and 3 matches the kitten
}

struct CursorTracker {
    idle_start: Option<Instant>,
    idle_duration: Duration,
    last_cursor_position: Option<Pos2>,
}

impl Default for CursorTracker {
    fn default() -> Self {
        CursorTracker {
            idle_start: Some(Instant::now()),
            idle_duration: Duration::ZERO,
            last_cursor_position: None,
        }
    }
}
//Implement copy and clone traits just to avoid errors when defining LaserPoint struct
impl Copy for CursorTracker {}

impl Clone for CursorTracker {
    fn clone(&self) -> Self {
        *self
    }
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

    fn change_target_kitten(&mut self, rect: Rect) {
        let bottom_right = rect.max;
        //Reduce the probability of the target moving to let the pointer stop less
        if fastrand::f32() > 0.70 {
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
        screen_state: ScreenState::Menu,
        cursor_tracker: CursorTracker::default(),
        }
    }
}

impl eframe::App for LaserPointer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //Draw a top panel to go back to the menu
        if let ScreenState::Algorithm(_)=self.screen_state {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Go back to menu").clicked() {
                            self.screen_state=ScreenState::Menu;
                        }
                    });
                });
            });
        }
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.screen_state{
                ScreenState::Menu=> {
                    ui.heading("Select a cat mode");
                    if ui.button("Restless cat mode").clicked() {
                        self.screen_state=ScreenState::Algorithm(1);
                    }
                    if ui.button("Normal cat mode").clicked() {
                        self.screen_state=ScreenState::Algorithm(2);            
                    }
                    if ui.button("Kitten mode").clicked() {
                        self.screen_state=ScreenState::Algorithm(3);            
                    }
                },
                ScreenState::Algorithm(1)=> {
                    ui.heading("Restless cat mode");
                    let current_mouse_position=ctx.input(|i| i.pointer.hover_pos());
                    // Check if the cursor is idle for more than 0.5 seconds
                    if let Some(pos) = current_mouse_position {
                        if let Some(last_pos)=self.cursor_tracker.last_cursor_position {
                            if last_pos!=pos {
                                //Mouse moved reset timer
                                self.cursor_tracker.idle_duration=Duration::ZERO;
                                self.cursor_tracker.idle_start=Some(Instant::now());
                            } else if let Some(start)=self.cursor_tracker.idle_start {
                                self.cursor_tracker.idle_duration=start.elapsed();
                            }
                        }
                        self.cursor_tracker.last_cursor_position=Some(pos);
                    }
                    let rect = ctx.screen_rect();
                    // If the idle time exceeds 0.5 seconds, set the speed to Still (laser pointer stops moving)
                    if self.cursor_tracker.idle_duration.as_secs_f64() > 0.5 {
                        self.speed = Speed::Still;
                    } else {
                        self.speed = Speed::Fast;
                        self.try_change_target(rect);
                        self.move_self();
                    }
                                    
                    let screen_size = Vec2 {
                            x: rect.width(),
                            y: rect.height()
                    };
                    let (_, painter) = ui.allocate_painter(screen_size,Sense::hover());
                    let LaserPointer { x, y, .. } = self;
                    let Pos2 { x: x2, y: y2 } =ctx.pointer_hover_pos().unwrap_or_default();
                    if (*x - x2).abs() < 40.0 && (*y - y2).abs() < 40.0 {
                            //self.try_change_target(rect);
                            //self.move_self();
                            self.random_movement(70.0);
                    } 
                    painter.circle_filled(Pos2::from(*self), 20.0, Color32::RED);
                    //Display the invisible target position
                    painter.circle_filled(self.imaginary_target, 10.0, Color32::GREEN);
                },
                ScreenState::Algorithm(2)=> {
                    ui.heading("Normal cat mode");
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
                },
                ScreenState::Algorithm(3)=> {
                    ui.heading("Kitten mode");
                    let rect = ctx.screen_rect();
                    //self.try_change_speed();
                    self.speed=Speed::CrazyFast;
                    //self.try_change_target(rect);
                    self.change_target_kitten(rect);
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
                },
                _ => {},
            }
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