use std::{
    env::current_dir,
    fs::{read_dir,read_to_string, write, /*Metadata,*/ DirEntry},
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
    file_loc: PathBuf,//Captures the path of the clicked file
    edited_content: String,//Capture the content of the modification in the right panel
    is_saved: bool, //track whether the file is saved or not
    show_unsaved: bool, //track the condition of whether the prompt window about saving should be displayed
    pending_file_loc: Option<PathBuf>,//Stores the file location where the user wants to switch
    }

impl DirectoryApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            //This is a good example of where we might want to keep .unwrap()—or turn it into .expect()—because if
            //there is a problem getting the current directory on startup, the whole app should crash
            //to allow us to try to fix what’s wrong.
            file_content: String::new(),
            edited_content: String::new(),
            current_dir: current_dir().unwrap(),
            file_loc: PathBuf::new(),
            is_saved: true,
            show_unsaved: false,
            pending_file_loc: None,
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
                eprintln!("Error reading file: {}", e);
            }
        }
    }
    fn save_file(&mut self) {
        // if let expression was refactored here
        write(&self.file_loc, &self.edited_content)
        .map(|_| self.is_saved=true)
        .unwrap_or_else(|e| {eprintln!("Error saving file: {}", e)})
        /*if let Err(e) = write(&self.file_loc, &self.edited_content) {
            eprintln!("Error saving file: {}", e);
        } else {self.is_saved=true;}*/
    }
    //new method to refactor how we handle each entry when reading the current directory
    fn process_entry(&mut self, entry: DirEntry, ui: &mut egui::Ui) {
        let metadata=  match entry.metadata() {
            Ok(meta) => meta,
            Err(_)=> return
        };
        let name= match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => return,
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
                //question mark operator. 
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
                        // Save the file content and load the next file
                        //Refactoring of "if let" expression made here
                        write(&self.file_loc, &self.edited_content)
                        .map(|_| {self.is_saved=true})
                        .unwrap_or_else(|e| {eprintln!("Error saving file: {}", e)});
                        /*if let Err(e) = std::fs::write(&self.file_loc, &self.edited_content) {
                            eprintln!("Error saving file: {}", e);
                        } else {
                            self.is_saved = true;
                        }*/
                        self.show_unsaved = false;
                        //Refactoring of "if let" expression made here
                        self.pending_file_loc.take()
                        .map(|new_file_loc| {self.update_file_content(new_file_loc)});
                        /*if let Some(new_file_loc) = self.pending_file_loc.take() {
                            self.update_file_content(new_file_loc);
                        }*/
                    }
                    if ui.button("Discard Changes").clicked() {
                        self.show_unsaved = false; // Close the unsaved prompt
                        // Discard changes and load the content of the next file while refactoring the "if let" expression
                        self.pending_file_loc.take()
                        .map(|new_file_loc| {self.update_file_content(new_file_loc)});
                        /*if let Some(new_file_loc) = self.pending_file_loc.take() {
                            self.update_file_content(new_file_loc);
                        }*/
                        
                    }
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| { 
            let mut is_checked = IS_CHECKED.lock().unwrap();
            ui.horizontal(|ui| {
                ui.add(Checkbox::new(&mut is_checked, "Show file content")); 
                if *is_checked && !self.file_content.is_empty() {
                    if ui.button("Save").clicked() {
                            self.save_file();
                    }
                }
            });
        });    
        if *IS_CHECKED.lock().unwrap() {
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
    "File explorer",
    native_options,
    Box::new(|cc| Ok(Box::new(DirectoryApp::new(cc)))),
    );
}