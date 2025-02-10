/* You can contribute to this solution by giving for example the user the choice to change the language of API in every loop
 */

use crossterm::{
    event::{read, Event, KeyEventKind, KeyCode},
    execute,
    terminal::{Clear,ClearType},
};

use serde::{Deserialize, Serialize};

use reqwest::{blocking::get, StatusCode};
use std::{error::Error, fmt::Debug, io::{stdout, stdin}, collections::HashMap};
use csv::Reader;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
    widgets::Wrap
};

#[derive(Debug, Deserialize, Default, Serialize)]
struct CurrentArticle {
    title: String,
    description: String,
    extract: String,
}

#[derive(Debug, Default)]
struct App {
    search_string: String,
    current_article: CurrentArticle,
    last_error: Option<String>,//This is used to get the last error and store later in the get_article function
    // If we don't store and rely only on printing errors, they will never be printed as we clear them out before printing
    // as you can notice in the main function
}

impl App {
    fn get_article(&mut self, url_wihtout_request: String) -> Result<(), Box<dyn Error>> {
        let url=format!("{url_wihtout_request}/{}", self.search_string);
        //HTTP errors are also part of the response token. you will have to match the status codes  
        //of the get method in reqwest crate when a response is returned
        match get(&url) {
            Ok(response) => {
                match response.status() {
                    StatusCode::OK => {
                        let text=response.text()?;
                        if let Ok(article) = serde_json::from_str::<CurrentArticle>(&text) {
                            self.current_article=article;
                            self.last_error = None;
                    } else {self.last_error =Some("Error! Received an invalid json format".to_string());}
                },
                    StatusCode::BAD_REQUEST => {self.last_error =Some("Error 400: Bad request".to_string());},
                    StatusCode::UNAUTHORIZED => {self.last_error =Some("Error 401: Unauthorized".to_string());},
                    StatusCode::FORBIDDEN => {self.last_error=Some("Error 403: Forbidden".to_string());},
                    StatusCode::NOT_FOUND => {self.last_error=Some("Error 404: Not Found".to_string());},
                    StatusCode::TOO_MANY_REQUESTS => {
                        //The API documentation says that number of requests per day is limited
                        self.last_error=Some("Error 429: Too Many Requests".to_string());},
                    StatusCode::REQUEST_TIMEOUT => {self.last_error=Some("Error 408: Request Timeout".to_string());},
                    StatusCode::INTERNAL_SERVER_ERROR => {self.last_error=Some("Error 500: Internal Server Error".to_string());},
                    StatusCode::BAD_GATEWAY => {self.last_error=Some("Error 502: Bad Gateway".to_string());},
                    StatusCode::SERVICE_UNAVAILABLE => {self.last_error=Some("Error 503: Service Unavailable".to_string());},
                    StatusCode::GATEWAY_TIMEOUT => {self.last_error=Some("Error 504: Gateway Timeout".to_string());},
                    _ => {//We will not mind about other HTTP errors 
                        self.last_error=Some("Error: Unknown HTTP Error".to_string());},
                }
            },
            //Now this kind of errors means that there was a problem sending a request
            Err(error) => {
                if error.is_connect() {self.last_error=Some("No internet connection".to_string());} else if error.is_timeout() {
                    self.last_error=Some("Request timed out".to_string());} else {
                        self.last_error=Some(format!("Error: {}", error));
                }
                    
                }
            }
            Ok(())
        }
    fn draw_response(&self) -> Result<(), anyhow::Error>{
            let stdout=stdout();
            let backend=CrosstermBackend::new(&stdout);
            let mut terminal=Terminal::new(backend)?;
            terminal.draw( |f| {
                let layout=Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(10),
                Constraint::Percentage(20), Constraint::Percentage(50),Constraint::Percentage(10)])
                .split(f.area());
            let search_area=layout[0];
            let article_title_area=layout[1];
            let article_description_area=layout[2];
            let article_extract_area=layout[3];
            let error_area=layout[4];

            let search_block=Block::default().title("Searching for").borders(Borders::ALL);
            let title_block=Block::default().title("Article title").borders(Borders::ALL);
            let description_block=Block::default().title("Article description").borders(Borders::ALL);
            let extract_block=Block::default().title("Article extract").borders(Borders::ALL);
            let error_block=Block::default().title("Error").borders(Borders::ALL);

            let search_text=Paragraph::new(self.search_string.clone())
            .block(search_block)
            .wrap(Wrap {trim: false}); // add the wrapping in order for the text no to overflow the borders
            let title_text=Paragraph::new(self.current_article.title.clone())
            .block(title_block)
            .wrap(Wrap {trim: false});
            let description_text=Paragraph::new(self.current_article.description.clone())
            .block(description_block)
            .wrap(Wrap {trim: false});
            let extract_text=Paragraph::new(self.current_article.extract.clone())
            .block(extract_block)
            .wrap(Wrap {trim: false});
            let error_text=Paragraph::new(self.last_error.as_deref().unwrap_or(""))
            .block(error_block)
            .wrap(Wrap {trim: false});    


            f.render_widget(search_text, search_area);
            f.render_widget(title_text, article_title_area);
            f.render_widget(description_text, article_description_area);
            f.render_widget(extract_text, article_extract_area);
            f.render_widget(error_text, error_area);
        })?;
        Ok(())
        
}
}
/*impl std::fmt::Display for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
        "
            Searching for: {}
        Title: {}
        ----------
        Description: {}
        ----------
        {}
        {}",
        self.search_string,
        self.current_article.title,
        self.current_article.description,
        self.current_article.extract,
        self.last_error.as_deref().unwrap_or("")//use as_deref method to convert Option<String> to Option<&str> as the
        // does not implement the Display trait
        )
    }
}
*/
//Remove the language code to let the choice to the user hereafter 
const URL_WITHOUT_LANGUGAGE: &str= "wikipedia.org/api/rest_v1/page/summary";
// Function to load the "wikipedia_languages" file and form the corresponding Hashmap to use it later on
fn load_languages() -> Result<HashMap<String,String>, Box<dyn Error>> {
    let mut langs = HashMap::new();
    let mut reader = Reader::from_path("wikipedia_languages.csv")?;
    //We skip the first record (headers)
    for result in reader.records().skip(1) {
        let record = result?;
        if let (Some(lang), Some(code)) = (record.get(0), record.get(1)) {
            langs.insert(lang.to_string(), code.to_string());
        }
    }
    Ok(langs)
}

fn main() -> Result<(), Box<dyn Error>> {
    let language_map = load_languages()?;
    let mut app=App::default();
    let mut input_language = String::new();
    println!("Please type the language of the API [English]");
    
    stdin().read_line(&mut input_language)?;
    let language = input_language.trim_end().to_lowercase();// We should only trim the end of the string 
    //to remove the characters '\n' added when you press Enter key
    let mut url_without_request =format!("https://en.{URL_WITHOUT_LANGUGAGE}");//Default to English
    if let Some(code) = language_map.get(&language) {
        url_without_request = format!("https://{}.{URL_WITHOUT_LANGUGAGE}", code);
        println!("Language found: {}", language);
    } else {
        println!("Language not found. Using English as default");
    }
    loop {
        //println!("{}", app);
        let _drawing=app.draw_response();
        _drawing?;

        if let Event::Key(key_event)=read()? {
            if key_event.kind==KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Backspace => {
                        app.search_string.pop();
                    },
                    KeyCode::Esc => app.search_string.clear(),// I think it would have been better to break the loop here
                    // But let's respect the choice of the author. Unfortunately, quitting the program is done
                    // by pressing an ugly Ctrl+C
                    KeyCode::Enter =>  app.get_article(url_without_request.clone())?,
                    KeyCode::Char(c) => app.search_string.push(c),
                    _ => {}
                }
            }
            execute!(stdout(), Clear(ClearType::All))?;
        }
    }
}
