use crossterm::{event::{self, poll, read, Event, KeyCode, KeyEventKind}, execute,
    terminal::{ disable_raw_mode, enable_raw_mode, Clear, ClearType}};
use std::{ fmt::Debug, io::stdout, error::Error, thread::sleep, time::{Duration, Instant}, borrow::Cow};
use chrono::{offset::{Utc, FixedOffset}, naive::NaiveDateTime};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, Axis, Chart, Dataset, GraphType},
    style::{Style, Stylize},
    symbols,
    text::Span,
    Terminal
};
use serde::{Deserialize, Serialize};
use reqwest::{blocking::get, StatusCode};

struct Stopwatch {
    now: Instant,
    state: StopwatchState,
    display: Cow<'static, str>, // cast into a Cow to optimize the performance later enabling static allocation
    paused_time: Duration,// Track the pause duration to adjust the stopwatch when there is a change of focus
}

const URL: &str = "https://api.open-meteo.com/v1/forecast?";



#[derive(Debug, Deserialize, Default, Serialize)]
struct WeatherForecast {
    latitude: f64,
    longitude: f64,
    timezone: String,
    hourly: HourlyData,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct HourlyData {
    time: Vec<String>,          // Vec of Strings for time
    temperature_2m: Vec<f64>,   // Vec of f64 for temperatures
}

#[derive(Debug, Default)]
struct App {
    search_latitude: f64,
    search_longitude: f64,
    forecast_weather: WeatherForecast,
    last_error: Option<String>,
}
impl App{
    fn new(search_latitude: f64, search_longitude: f64) -> Result<Self, std::io::Error> {
        let forecast_weather=WeatherForecast::default();
        Ok(Self{
            search_latitude: search_latitude,
            search_longitude: search_longitude,
            forecast_weather,
            last_error: None,
        })
    }
    fn get_weather(&mut self) -> Result<(), Box<dyn Error>>{
        let url=format!("{URL}latitude={}&longitude={}&hourly=temperature_2m", &self.search_latitude, 
        &self.search_longitude);
        match get(&url) {
            Ok(response) => {
                match response.status() {
                    StatusCode::OK => {
                        let text=response.text()?;
                        if let Ok(weather) = serde_json::from_str::<WeatherForecast>(&text) {
                            self.forecast_weather=weather;
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
            Err(error) => {
                if error.is_connect() {self.last_error=Some("No internet connection".to_string());} else if error.is_timeout() {
                    self.last_error=Some("Request timed out".to_string());} else {
                        self.last_error=Some(format!("Error: {}", error));
                }
                    
                }
            }
            Ok(())
        }
        // function to prepare the format of the data to be incorporated into the ratatui chart object
        fn get_dataset(&self) -> Vec<(f64,f64)>{
            //Get access to the time vec from the token
            let time=&self.forecast_weather.hourly.time ;
            //Get access to the temperature_2m vec from the token
            let temperature_2m=&self.forecast_weather.hourly.temperature_2m;
            //Choose a reference datetime to proceed with the conversion of dates to floats 
            let reference_datetime=NaiveDateTime::parse_from_str(&time[0], "%Y-%m-%dT%H:%M")
            .expect("Invalid datetime format");
            let temperature_dataset: Vec<(f64,f64)>=time.iter()
            .zip(temperature_2m.iter())
            .map(|(t,&temp)| {
                let parsed_datetime= NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M")
                .expect("Invalid datetime format");
                // Calculate the difference between the parsed datetime and reference datetime
                let duration = parsed_datetime.signed_duration_since(reference_datetime);
                //let parsed_datetime=NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M")
                //.expect("Invalid datetime format");
                let x=duration.num_seconds() as f64 /3600.0;
                (x,temp)
            }).collect();
            temperature_dataset
        }
        // Function to prepare the labels for the x axis to incorporate it into the Axis object of ratatui crate.
        fn get_x_labels(&self) -> Vec<Span> {
            let times = &self.forecast_weather.hourly.time;
            if times.is_empty() {
                return vec![];
            }
            // Let's choose the first, middle, and last labels. Otherwise the labels we will get a lot of labels
            // which cannot be displayed later in the graph due to the overlapping of text.
            let first = times.first().unwrap();
            let middle = times.get(times.len()/2).unwrap();
            let last = times.last().unwrap();
            vec![Span::raw(first), Span::raw(middle), Span::raw(last)]
        }
        // Function to prepare the labels for the y axis to incorporate it into the Axis object of ratatui crate.
        fn get_y_labels(&self) -> Vec<Span> {
            // Prepare the labels for the sorting operation
            let mut y_labels: Vec<f64>=self.forecast_weather.hourly.temperature_2m.iter().cloned().collect();
            // Sort the labels in order to display them correctly in the y axis of the chart
            y_labels.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let sorted_labels: Vec<Span> = y_labels.iter().map(|&label| Span::raw(format!("{:.1}", label)))
            .collect();
            sorted_labels
        }
}
enum StopwatchState {
    NotStarted,
    Running,
    Done,
}

impl Stopwatch {
    fn new()->Self {
        Self {
            now: Instant::now(),
            state: StopwatchState::NotStarted,
            display: Cow::Borrowed("0:00:00"),
            paused_time: Duration::ZERO,
        }
    }
    // Convert the String output to a string literal with a static lifetime to enable caching and use
    // Cow to lets you return a &str if you don’t need an owned String
    fn get_time(&self) -> Cow<'static ,str> { 
        use StopwatchState::*;
        match self.state {
            NotStarted => Cow::Borrowed("00:00:00"),
            Running => {
                let mut elapsed=self.now.elapsed().as_millis()+self.paused_time.as_millis();
                let minutes=elapsed / 60000;
                elapsed-=minutes*60000;
                let seconds = elapsed/1000;
                elapsed-=seconds*1000;
                let split_seconds=elapsed/10;
                // Show at least two digits in the stopwatch 
                Cow::Owned(format!("{:02}:{:02}:{:02}",minutes,seconds,split_seconds))
            },
            Done => Cow::Owned(self.display.to_string()),
        }
    }
    fn next_state(&mut self) {
        use StopwatchState::*;
        match self.state {
            NotStarted => {
                self.now=Instant::now();
                self.state=Running;
            },
            Running => {
                self.display=self.get_time();
                self.state=Done;
            },
            Done => {
                self.state=NotStarted;
                self.paused_time = Duration::ZERO;
            },
        }
    }
    //pause method to pause the stopwatch when the focus is lost
    fn pause(&mut self) {
        if let StopwatchState::Running = self.state {
            self.paused_time += self.now.elapsed();
            self.state=StopwatchState::NotStarted;
            
        }
    }
    //resume method to resume the stopwatch count when the focus is gained
    fn resume(&mut self) {
        if let StopwatchState::NotStarted = self.state {
            self.now=Instant::now();
            self.state=StopwatchState::Running;
        }
    }
}


// Define this function just for readability in the main function below.
fn block_with(input: &str) -> Block {
    Block::default().title(input).borders(Borders::ALL)
}
// Define this function just for readability in the main function below.
fn time_pretty(h: i32) -> String { // I have changed the name og the function to handle other time zones.
    let offset = FixedOffset::east_opt(h * 3600).unwrap_or_else(|| FixedOffset::east_opt(0)
                                .unwrap()); // Ensure valid offset
    Utc::now().with_timezone(&offset).format("%Y/%m/%d %H:%M:%S").to_string()
}
// Using anyhow::Error to propagate multiple types of errors
fn main() -> Result<(), anyhow::Error> {
    let stdout=stdout();
    let backend=CrosstermBackend::new(&stdout);
    // Latitude and longitude of London are entered below.
    let mut app_london=App::new(51.509865, -0.118092)?;
    let _weather_london=app_london.get_weather();
    let data=app_london.get_dataset();
    // Define the bounds of the chart to display.
    let (x_min, x_max) = data.iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (x, _)| {
            (min.min(*x), max.max(*x))
        });
    
    let (y_min, y_max) = data.iter()
    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (_, y)| {
            (min.min(*y), max.max(*y))
        });
    
    // The ratatui terminal takes a crossterm backend.
    let mut terminal=Terminal::new(backend)?;
    let mut stopwatch=Stopwatch::new();
    // Enable raw mode and focus event detection
    enable_raw_mode()?;
    execute!(&stdout, event::EnableFocusChange, Clear(ClearType::All))?;
    loop{
        if poll(Duration::from_millis(0))? {
            let event=read()?;
            match event {
                /* Warning: Capturing FocusGained and FocusLost events are not warranted for all terminals.
                In my case, I tested this program with cmd terminal included in vs code but both events were
                not captured. But when I used the standalone version of the cmd it worked perfectly.
                This might be a lesson to take: Whenever you use crossterm crate try to test your program
                in a standalone terminal*/
                Event::FocusLost => {
                    stopwatch.pause();
                },

                Event::FocusGained => {
                    stopwatch.resume();
                },
                Event::Key(key_event) => {
                    match (key_event.code, key_event.kind) {
                        (KeyCode::Enter, KeyEventKind::Press) =>{
                            stopwatch.next_state();
                            println!("Enter key pressed")
                        },
                        (KeyCode::Esc, KeyEventKind::Press) =>{
                            break;// we absolutely must add a break condition to the infinite loop in order to disable the raw mode later
                        },
                        _ => {},
                    }
                },
                _ => {}
            }
        }
        terminal.draw( |f| {
            let layout=Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());
        let stopwatch_area=layout[0];
        let time_area=layout[1];
        // Split once again the time_area frame. I chose London as required then Kyiv and New York city.
        let split_time_area=Layout::default().direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10), // Allocate space for the first paragraph
            Constraint::Percentage(70), // Allocate space for the second paragraph
            Constraint::Percentage(10), // Allocate space for the third paragraph
            Constraint::Percentage(10), // Allocate space for the fourth paragraph
        ])
        .split(time_area);
        let london_time_area=split_time_area[0];
        let london_chart_area=split_time_area[1];
        let kyiv_time_area=split_time_area[2];
        let nyork_time_area=split_time_area[3];    

        let stopwatch_block=block_with("Stopwatch");
        let utc_time_block=block_with("London Time");
        let london_temp_block=block_with("London Temperature");
        let kyiv_time_block=block_with("Kyiv Time");
        let nyork_time_block=block_with("New York Time");  

        let stopwatch_text=Paragraph::new(stopwatch.get_time()).block(stopwatch_block);
        let london_time_text=Paragraph::new(time_pretty(0)).block(utc_time_block);
        let kyiv_time_text=Paragraph::new(time_pretty(5)).block(kyiv_time_block);
        let nyork_time_text=Paragraph::new(time_pretty(-5)).block(nyork_time_block);

        let dataset=vec![Dataset::default()
        .name("Time vs Temperature")
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().cyan())
        .data(&data)
        ];
        
        let x_labels=app_london.get_x_labels();
        let y_labels=app_london.get_y_labels();
        let x_axis=Axis::default()
        .title("Time".red())
        .style(Style::default().white())
        .bounds([x_min, x_max])
        .labels(x_labels);

        let y_axis=Axis::default()
        .title("Temperature (°C)".green())
        .style(Style::default().white())
        .bounds([y_min, y_max])
        .labels(y_labels);

        let chart=Chart::new(dataset)
        .block(london_temp_block)
        .x_axis(x_axis)
        .y_axis(y_axis);

        f.render_widget(stopwatch_text, stopwatch_area);
        f.render_widget(london_time_text, london_time_area);
        f.render_widget(chart, london_chart_area);
        f.render_widget(kyiv_time_text, kyiv_time_area);
        f.render_widget(nyork_time_text, nyork_time_area);

        })?;
        //The terminal is going to loop as fast as it possibly can, so let’s put it to sleep each
        //time to keep the screen from flickering. Using .sleep() can be a bad idea in complex and async code,
        //but we are just running a little terminal app on a single thread.
        sleep(std::time::Duration::from_millis(10));
        // Ratatui has a convenience method called .clear(), so we don’t need to use a crossterm
        //command to clear the screen anymore.
        terminal.clear()?;
    }
    disable_raw_mode()?;
    execute!(&stdout, event::DisableFocusChange)?;
    Ok(())
}
