//This code has been done after completing the answer to question #5 of the file navigator project
use std::{
    env::current_dir,
    fs::{read_dir,read_to_string, write, DirEntry},
    path::PathBuf,
    };
use eframe::egui;
use fastrand;
//RichText is used in egui if you want to change the text of a widget, and Color32 allows us to choose a color.
use egui::{Checkbox, Color32, Pos2, Rect, RichText, TextEdit};
//The app so far holds a PathBuf that we will use .push() and .pop() on.
struct DirectoryApp {
    file_content: String,
    current_dir: PathBuf,
    file_loc: PathBuf,//Captures the path of the clicked file
    edited_content: String,//Capture the content of the modification in the right panel
    is_saved: bool, //track whether the file is saved or not
    show_unsaved: bool, //track the condition of whether the prompt window about saving should be displayed
    pending_file_loc: Option<PathBuf>,//Stores the file location where the user wants to switch
    error_messages: Vec<String>, // New field to store error messages
    show_errors: bool, // New field to track if errors should be shown
    show_error_window: bool, // New field to track whether the error window should be shown
    is_checked: bool, //New field to track whether the save file checkbox is activated
    laser_pointer: LaserPointer,
    laser_pointer_checked: bool,//New field to track whether the laser pointer is activated
    }

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

impl From<LaserPointer> for Pos2 {
    fn from(pointer: LaserPointer) -> Self {
        Pos2 {
            x: pointer.x,
            y: pointer.y,
        }
    }
}

impl LaserPointer {
    fn new() -> Self {
        Self {
            x: 50.0,
            y: 50.0,
            speed: Speed::default(),
            imaginary_target: Pos2 { x: 50.0, y: 50.0 },
        }
    }

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

    fn move_self(&mut self) {
        let x_from_target = self.imaginary_target.x - self.x;
        let y_from_target = self.imaginary_target.y - self.y;
        self.x += fastrand::f32() * x_from_target * self.change_speed();
        self.y += fastrand::f32() * y_from_target * self.change_speed();
    }

    fn update(&mut self,ctx: &egui::Context) {
        ctx.request_repaint(); 

        // Get a painter for the foreground layer (this will overlay everything)
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground, 
            egui::Id::new("laser_pointer")
        ));
            self.try_change_speed();
            let rect = ctx.screen_rect();
            self.try_change_target(rect);
            self.move_self();
            let LaserPointer { x, y, .. } = self;
            let Pos2 { x: x2, y: y2 } =ctx.pointer_hover_pos().unwrap_or_default();
            if (*x - x2).abs() < 20.0 && (*y - y2).abs() < 20.0 {
                self.random_movement(50.0);
            }
            painter.circle_filled(Pos2::from(*self), 20.0, Color32::RED);
    }
}

impl DirectoryApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            file_content: String::new(),
            edited_content: String::new(),
            current_dir: current_dir().unwrap(),
            file_loc: PathBuf::new(),
            is_saved: true,
            show_unsaved: false,
            pending_file_loc: None,
            error_messages: Vec::new(), // Initialize error messages
            show_errors: false,
            show_error_window: false,
            is_checked: false,
            laser_pointer:LaserPointer::new(),
            laser_pointer_checked: false,
        }
    }

    fn load_file(&mut self,file_path: PathBuf) {
        if !self.is_saved {
            self.show_unsaved = true;
            self.pending_file_loc = Some(file_path);
        } else {
            self.update_file_content(file_path);
        }
    }

    fn update_file_content(&mut self,file_path: PathBuf) {
        match read_to_string(&file_path) {
            Ok(content) => {
                self.file_content = content.clone();
                self.edited_content = content;
                self.file_loc = file_path;
                self.is_saved = true;
            },
            Err(e) => {
                if self.show_errors {
                    self.error_messages.push(format!("Error reading file: {}", e));
                }
            }
        }
    }

    fn save_file(&mut self) {
        write(&self.file_loc, &self.edited_content)
        .map(|_| self.is_saved=true)
        .unwrap_or_else(|e| {
            self.error_messages.push(format!("Error reading file name: {:?}", e));
        })
    }

    fn process_entry(&mut self, entry: DirEntry, ui: &mut egui::Ui) {
        let metadata=  match entry.metadata() {
            Ok(meta) => meta,
            Err(e)=> {
                if self.show_errors{
                self.error_messages.push(format!("Error reading metadata: {}", e));
                }
                return;
            }
        };
        let name= match entry.file_name().into_string() {
            Ok(n) => n,
            Err(e) => {
                if self.show_errors{
                self.error_messages.push(format!("Error reading file name: {:?}", e));
                }
                return;
            }
        };
        match metadata.file_type() {
            t if t.is_dir() => {if ui.button(RichText::new(&name).color(Color32::GRAY)).clicked(){
                self.current_dir.push(&name);
                }
            },
            t if t.is_file() => {if ui.button(RichText::new(&name).color(Color32::GOLD))
                .clicked(){
                    let new_file_loc: PathBuf = self.current_dir.join(&name);
                    // Check if there are unsaved changes before switching files
                    self.load_file(new_file_loc);
                }
            },
            _ => {ui.label(format!("{:?}", metadata.file_type()));},
        }
    }

}

impl eframe::App for DirectoryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
                // Add space to prevent overlap with the top panel which will include the checkbox
            ui.add_space(25.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                if ui.button(" .. ").clicked() {
                    self.current_dir.pop();
                }
                if let Ok(read_dir) = read_dir(&self.current_dir) {
                //Note that here we are using .flatten() to ignore anything inside the read_dir() 
                //method that returns an Err.
                    for entry in read_dir.flatten() {
                        //We get the metadata and file/directory name. With the metadata we can see
                        //whether we have a file or a directory.
                        self.process_entry(entry, ui);
                        }
                    }
            });
        });
        if self.show_unsaved {
            // Show unsaved changes prompt window
            let mut show_unsaved = self.show_unsaved;
            egui::Window::new("File is unsaved. Do you want to save changes?")
                .open(&mut show_unsaved)
                .show(ctx, |ui| {
                    if ui.button("Save Changes").clicked() {
                        write(&self.file_loc, &self.edited_content)
                        .map(|_| {self.is_saved=true})
                        .unwrap_or_else(|e| {
                            self.error_messages.push(format!("Error saving written file: {:?}", e));
                        });
                        self.show_unsaved = false;
                        //Refactoring of "if let" expression made here
                        self.pending_file_loc.take()
                        .map(|new_file_loc| {self.update_file_content(new_file_loc)});
                    }                    
                    if ui.button("Discard Changes").clicked() {
                        self.show_unsaved = false; // Close the unsaved prompt
                        self.pending_file_loc.take()
                        .map(|new_file_loc| {self.update_file_content(new_file_loc)});                        
                    }
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| { 
            ui.horizontal(|ui| {
                ui.add(Checkbox::new(&mut self.is_checked, "Show file content")); 
                
                if self.is_checked && !self.file_content.is_empty() {
                    if ui.button("Save").clicked() {
                            self.save_file();
                    }
                }

                if ui.add(Checkbox::new(&mut self.show_errors, "Show Error Window")).clicked(){
                    if !self.error_messages.is_empty() {
                        self.show_errors = true; 
                    } else {
                        self.show_error_window = false; 
                    }
                };
                ui.add(Checkbox::new(&mut self.laser_pointer_checked, "Show laser pointer"));
            });
        });    
        if self.laser_pointer_checked {
            self.laser_pointer.update(ctx);
        }
        if self.is_checked {
            //If the checkbox is checked, we display the file content in a new panel on the right side.
            let width = ctx.screen_rect().max.x / 2.0;
            if !self.file_content.is_empty() {
                egui::SidePanel::right("Text viewer")
                .min_width(width)
                .show(ctx, |ui| {
                    ui.add_space(25.0);
                    let response = ui.add(TextEdit::multiline(&mut self.edited_content)
                    .desired_width(width));
                    // Detect changes and update is_saved
                    if response.changed() {
                        self.is_saved = false;
                    }
                });
            }
        }
    }

}

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
    "File explorer with laser pointer",
    native_options,
    Box::new(|cc| Ok(Box::new(DirectoryApp::new(cc)))),
    );
}