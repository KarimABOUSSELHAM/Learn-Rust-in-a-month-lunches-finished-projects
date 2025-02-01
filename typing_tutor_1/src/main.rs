//This main file tries to answer the first question about giving the time which the test takes and 
// giving the typing speed in wpm
use crossterm::{event::{ read, Event, KeyCode, KeyEventKind}, terminal::{Clear, ClearType}, execute};
use std::{fs::{read_to_string, read_dir}, io::stdout, io::stdin, time::Instant};
use ansi_term::Colour::Red;
use unicode_normalization::{char::compose, UnicodeNormalization};
use std::io;
use regex::Regex;
struct App {
    file_content: String,
    user_input: String,
    hidden_input: String,// This is a flag to save the code to compose accented letters with unicode
}

impl App {
    fn new(file_name: &str) -> Result<Self, std::io::Error> {
        let file_content=read_to_string(file_name)?;
        Ok(Self {
            file_content,
            user_input: String::new(),
            hidden_input: String::new(),
        })
    }
}

fn implement_accented_word(char: char,hex: &String) -> Option<char> {
    let content = read_to_string("diacritical_marks.txt").ok()?;
    let accents_hex_codes: Vec<&str> = content.lines().collect();
    if accents_hex_codes.contains(&hex.as_str()){
        // Parse the string into a u32
        let unicode_value = u32::from_str_radix(&hex, 16).ok()?;
        // Convert the u32 into a char
        let char_output=char::from_u32(unicode_value).unwrap_or('\u{034F}');
        compose(char,char_output)
    } else {
        Some(char)
    }
}

fn typing_tutor_per_sample(file_name:&str) -> Result<(), std::io::Error> {
    let mut app= App::new(file_name)?;
    let before_test=Instant::now();
    // Variable to control the accented letters through the down key instead of alt key
    let mut down_pressed = false;
    // variable to control the cases of accented "e" letters obtained through unicode composition
    let re_contains_e= Regex::new(r"([eE])\p{M}*").unwrap();
    let e_vector_without_unicode=vec!['é','è','ê','Ê']; //This vec includes the instances of "e" which we can type directly from the keyboard
    loop {
        
        // Recall the unwrap method should not be used in production code because you could never predict the
        // outcome of the read() function.
        println!("{}",app.file_content);
        // The underscore shows the user where the cursor is.
        // Note the Backspace and Esc events will not work as expected in terminals other than Powershell
        // windows cmd. You can find more details in the docs page of KeyEventKind
        for (letter1,letter2) in app.user_input.chars().zip(app.file_content.chars()){
            if letter1==letter2{
                print!("{letter2}");
            } else {
                // show incorrect entries in red instead of just an asterisk
                // convert letter1 to string in order to implement the Into<Cow<'_, _>>` trait
                print!("{}",Red.paint(letter1.to_string()));
            }
        }
        println!("_");
        if let Event::Key(key_event)= read()? {
            if key_event.kind==KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Backspace => {
                        if down_pressed { 
                            // Remove from hidden_input if DOWN is pressed
                            app.hidden_input.pop();} else {
                        app.user_input.pop();}
                    },
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => {
                        // keep user_input from getting longer than the file_content string
                            if app.user_input.len() < app.file_content.len(){
                                if re_contains_e.is_match(&c.to_string()) || e_vector_without_unicode
                                .contains(&c){
                                    app.user_input.push('|');//I don't recommend to use a space instead, cause it will 
                                    //introduce a bias to the way the score and speed are computed
                                } else {
                                app.user_input.push(c);
                            }
                        }
                    },
                    KeyCode::Enter => {
                        let normalized_text:String=app.file_content.nfd().collect(); 
                        // Transform the file content according to the regex pattern in order to better compute the score
                        // of total_chars later on.
                        let transformed_file_content = re_contains_e
                            .replace_all(&normalized_text, |caps: &regex::Captures| {caps[1].to_lowercase()});
                        let total_chars=transformed_file_content.chars().filter(|b| *b!='e').count();
                        let total_right=app.user_input.chars().zip(app.file_content.chars())
                            .filter(|(a,b)| a==b && *a!='|').count();//letter e replaced by "|" should not be 
                        // taken into account. 
                        println!("you got {total_right} out of {total_chars}!");
                        let after_test=before_test.elapsed();
                        println!("The test took {:?}", after_test);
                        let typing_length=app.user_input.split_whitespace().count() as f32;
                        let typing_speed=typing_length / after_test.as_secs_f32()*60.0;
                        println!("Typing speed: {} wpm", typing_speed);
                        
                        return Ok(());
                    },
                    KeyCode::Down => {
                        loop{
                        if let Event::Key(hidden_key_event)= read()? {
                            if hidden_key_event.kind==KeyEventKind::Press {
                                match hidden_key_event.code {
                                    KeyCode::Char(c) => {
                                        app.hidden_input.push(c);
                                    },
                                    KeyCode::Enter => break,
                                    _ => {}
                                }
                            }
                            }
                    }
                    },
                    _ => {}
                }
            }
            // Handling the Down key release event
            if key_event.kind == KeyEventKind::Release {
                if down_pressed {
                    // On DOWN key release, compose the characters
                    if let Some(last_char) = app.user_input.pop() {
                        // Compose the character typed before pressing DOWN with the hidden_input
                        let accented_word:Option<char>  =implement_accented_word(last_char, &app.hidden_input);
                        if let Some(accented_char) = accented_word {
                            app.user_input.push(accented_char);
                        } else {app.user_input.push(last_char);}
                        app.hidden_input.clear(); // Reset hidden_input after composition
                    }
                    down_pressed = false; // Reset default state
                }
            }
            // Track if Down key is being pressed
            if key_event.code == KeyCode::Down {
                down_pressed = true;
            }
    }
            execute!(stdout(),Clear(ClearType::All))?;
        }
    Ok(())
}

//function to retrieve the file names of the parent folder which start with the "typing" pattern
fn get_typing_samples() -> io::Result<Vec<String>> {
    let files=read_dir(".")?
    .filter_map(|entry| entry.ok())
    .filter(|entry| {//functional programming to control the existence of the file
        match entry.file_type() {
            Ok(file_type) => file_type.is_file(),
            Err(_) => false,
        }
    })
    .filter_map(|entry| entry.file_name().to_str().map(String::from))
    .filter(|name| name.starts_with("typing"))//We admit here that all test files start with the word "typing".
    .collect();
    Ok(files)
}
fn main() {
    //First, retrieve the sample files
    match get_typing_samples() {
        // The case where there is no issue reading the sample files
        Ok(file_names)=>{
            if file_names.is_empty() {
                println!("No typing sample files found! Please add files starting with 'typing' to the home project directory.");
                return;
            }
            // The first file sample must be passed without question.
            let _=typing_tutor_per_sample(&file_names[0]);
            let mut index: usize=1;
            let sample_numbers=file_names.len(); //This is in case you add more typing sample files
            let mut input_string = String::new();
            loop {
                // Give the choice to the user to choose more typing tests
                println!("Do you want to take more tests? [Y/N]");
                input_string.clear();
                stdin().read_line(&mut input_string).unwrap();
                let input: String = input_string.trim().to_string();//We convert to string to let it possible to clear it
                match input.as_str() {
                    // Always clear the input typed by the user in order not to push characters after each loop
                    "Y" | "y" => {if index>=sample_numbers{
                        println!("Oops! There is no test sample anymore. See you later!");
                        input.to_string().clear();
                        break;
                    } else { 
                        let _=typing_tutor_per_sample(&file_names[index]);
                        index+=1;
                        input.to_string().clear();
                    }
                },
                    "N" | "n" => {
                        println!("Okay, see you later!");
                        input.to_string().clear();
                        break;
                    },
                    _ => { 
                        println!("Invalid input. Please enter Y or N.");
                        input.to_string().clear();
                }
                }
            }
        },
        // Case where there is an issue reading one of the sample files or more.
        Err(error) => println!("Error reading files: {}", error)
        }
}