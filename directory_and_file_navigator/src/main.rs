use std::{
    env::current_dir,
    fs::{read_dir,read_to_string},
    path::PathBuf,
    };
use lazy_static::lazy_static;
use std::sync::Mutex;
use eframe::egui;
//RichText is used in egui if you want to change the text of a widget, and Color32 allows us to choose a color.
use egui::{Color32, RichText, TextEdit, Checkbox};
//The app so far holds a PathBuf that we will use .push() and .pop() on.
struct DirectoryApp {
    file_content: String,
    current_dir: PathBuf,
    }

impl DirectoryApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            //This is a good example of where we might want to keep .unwrap()—or turn it into .expect()—because if
            //there is a problem getting the current directory on startup, the whole app should crash
            //to allow us to try to fix what’s wrong.
            file_content: String::new(),
            current_dir: current_dir().unwrap(),
        }
    }
}

// Use lazy_static to define a globally accessible, mutable static variable
lazy_static! {
    static ref IS_CHECKED: Mutex<bool> = Mutex::new(false); // Mutex is used to allow safe mutable access
}

impl eframe::App for DirectoryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add space to prevent overlap with the top panel which will include the checkbox
            ui.add_space(25.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
            //This part is pretty easy! Make a button and .pop() when it is clicked.
            if ui.button(" .. ").clicked() {
                self.current_dir.pop();
            }
            //Now we are going to work through the directory information.
            //The unwraps have been removed, but egui’s update() method doesn’t return a Result, so we can’t use the
            //question mark operator. The if let syntax is helpful here.
            if let Ok(read_dir) = read_dir(&self.current_dir) {
            //Note that here we are using .flatten() to ignore anything inside the read_dir() 
            //method that returns an Err.
                for entry in read_dir.flatten() {
                    //We get the metadata and file/directory name. With the metadata we can see
                    //whether we have a file or a directory.
                    if let Ok(metadata) = entry.metadata(){
                        if metadata.is_dir() {
                            if let Ok(dir_name) = entry.file_name().into_string(){
                    //We’ll make buttons with different text depending on whether we have a file or a
                    //directory. If we have a directory, clicking the button will .push() to the
                    //PathBuf and move us into that directory.
                    
                                if ui
                                .button(RichText::new(&dir_name).color(Color32::GRAY))
                                .clicked(){
                                    self.current_dir.push(&dir_name);
                                }
                            }
                    } else if metadata.is_file() {
                        if let Ok(file_name)=entry.file_name().into_string() {
                            if ui
                            .button(RichText::new(&file_name).color(Color32::GOLD))
                            .clicked()
                            {
                                if let Some(current_dir) =self.current_dir.to_str() {
                                    //Here is the part with the new PathBuf to get the
                                    //contents of the file if a file button has been clicked.
                                    //We then use read_to_string() to create a String to
                                    //hold the file content. If there is an error, it will show
                                    //the error information instead of the file content.
                                    let file_loc: PathBuf =[current_dir,&file_name].iter().collect();
                                    //Finally, this part displays a new panel on the side if the app holds 
                                    //any file content.
                                    let content =read_to_string(file_loc)
                                    .unwrap_or_else(|e| e.to_string());
                                self.file_content = content;
                                }
                            }
                        }
                    } else {
                        ui.label(format!("{:?}", metadata.file_type()));
                        }
                    }
                }
            }
        });
    });
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| { 
        let mut is_checked = IS_CHECKED.lock().unwrap();
        ui.add(Checkbox::new(&mut is_checked, "Show file content")); 
        if *is_checked {
            //If the checkbox is checked, we display the file content in a new panel on the right side.
            let width = ctx.screen_rect().max.x / 2.0;
            if !self.file_content.is_empty() {
                egui::SidePanel::right("Text viewer")
                .min_width(width)
                .show(ctx, |ui| {
                    ui.add_space(25.0);
                    ui.add(TextEdit::multiline(&mut self.file_content).desired_width(width));
                });
            }
        }
    });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
    "File explorer",
    native_options,
    Box::new(|cc| Ok(Box::new(DirectoryApp::new(cc)))),
    );
}