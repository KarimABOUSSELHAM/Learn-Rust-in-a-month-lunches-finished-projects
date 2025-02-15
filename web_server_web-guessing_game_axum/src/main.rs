// For the question #3, I used shuttle which is very easy to understand and implement
// You can find the steps to deploy the server with shuttle in the docs 
// I pasted this code into the shuttle project main.rs file to get a feeling of what has changed
// However, as far as the fourth question is concerned I didn't integrate the shuttle with the session handler. 
//For this latter I used hereafter "tower-sessions" framework which replaced "axum_sessions".

use axum::{extract::{Path/* , State*/}, routing::get, response::Redirect}; //Use State for questions #1, #2 and #3 and remove it for question #4
use tokio::net::TcpListener;
use std::{net::SocketAddr,sync::{Mutex,Arc}};
use fastrand;
use tower_sessions::{MemoryStore, SessionManagerLayer,Session, Expiry::OnInactivity};
use time::Duration;
use serde::{Serialize, Deserialize};

const RANDOM_WORDS: [&str; 6] =["MB", "Windy", "Gomes", "Johnny", "Seoul", "Interesting"];

/*static GAME: Mutex<GameApp> = Mutex::new(GameApp {
    current_word: String::new(),
    right_guesses: vec![],
    wrong_guesses: vec![],
});*/



#[derive(Clone, Debug, Serialize, Deserialize)]// Serialize and Deserialize are implemented function start_session
struct GameApp {
    current_word: String,
    right_guesses: Vec<char>,
    wrong_guesses: Vec<char>,
}

enum Guess {
    Right,
    Wrong,
    AlreadyGuessed,
    }



impl GameApp {
        fn new() -> Self {
            let mut game = Self {
                current_word: String::new(),
                right_guesses: vec![],
                wrong_guesses: vec![],
            };
            game.restart();
            game
        }
        fn restart(&mut self) {
        self.current_word =RANDOM_WORDS[fastrand::usize(..RANDOM_WORDS.len())].to_lowercase();
        self.right_guesses.clear();
        self.wrong_guesses.clear();
        }
        fn check_guess(&self, guess: char) -> Guess {
        if self.right_guesses.contains(&guess) || self.wrong_guesses.contains(&guess) {
            return Guess::AlreadyGuessed;
            }
        match self.current_word.contains(guess) {
            true => Guess::Right,
            false => Guess::Wrong,
            }
        }
        fn results_so_far(&self) -> String {
                let mut output = String::new();
                    for c in self.current_word.chars() {
                    if self.right_guesses.contains(&c) {
                    output.push(c)
                    } else {
                    output.push('*')
                    }
                }
            output
        }
        fn take_guess(&mut self, guess: String) -> String {
            let guess = guess.to_lowercase();
            let mut output = String::new();
            match guess {
                guess if guess.chars().count() == 1 => {
                let the_guess = guess.chars().next().unwrap();
                match self.check_guess(the_guess) {
                    Guess::AlreadyGuessed => {
                    output.push_str(&format!("You already guessed {the_guess}!\n"));
                        }
                    Guess::Right => {
                    self.right_guesses.push(the_guess);
                    output.push_str(&format!("Yes, it contains a {the_guess}!\n"));
                        }
                    Guess::Wrong => {
                    self.wrong_guesses.push(the_guess);
                    output.push_str(&format!("Nope, it doesn't contain a {the_guess}!\n"));
                        }
                    }
                output.push_str(&self.results_so_far());
                }
                guess => {
                    if self.current_word == guess {
                    output.push_str(&format!("You guessed right, it's {}!\n", self.current_word));
                    output.push_str("Let's play again!\n");
                    self.restart();
                    } else {
                    output.push_str(&format!( "Bzzt! It's not {guess}, it's {}.\n"
                    , self.current_word));
                    output.push_str("Time to move on to another word!\n");
                    self.restart();
                    }
                    
                }
            }
        output
        }
}

// Use Arc<Mutex<GameApp>> to replace the global static with the method .with_state
// We should extract a state, so we use State instead of the Path extractor here to wrap a new argument
// But for question #4 we need to remove the notion of Arc<Mutex<GameApp>> in order to avoid that all requests share the same game instance.
// We don't want after all a user to change the state of the game instance to other users connected to other sessions.
async fn get_res_from_arc_mutex(//State(_game): State<Arc<Mutex<GameApp>>>,//The argument left-hand is a part of the answer to question #1
session: Session,Path((_, guess)): Path<(String, String)> ) -> String { //Only destructure Path for question #4
    // Otherwise axum won't understand it has to handle two parameters from the URL, which are the session_id and the guess of the user.
    // The following commented block corresponds to the answer of question #1.
    // Uncomment it and comment the next block if you want to run question #1.
    /*let mut game=game.lock().unwrap();
    game.take_guess(guess.clone())*/

    //mutability in the statement below is what guarantees the game instance to change inside the game session and let take_guess method 
    // to work correctly.
    let mut game = session.get::<GameApp>("game_state")
    .await
    .unwrap_or(Some(GameApp::new()))
    .unwrap();
    let result = game.take_guess(guess);
    session.insert("game_state", game).await.unwrap();
    result
    
    }

// This function is used for question #4, uniquely.
// Notice we do not want the user to 
async fn start_session(session: Session) -> Redirect { //We use Redirect here to redirect the user of the session when he types something
    //like localhost:port to something like localhost:port/session_id/game as required in question #4
    if session.get::<String>("game_id").await.unwrap().is_none() {
        let new_game = GameApp::new();
        session.insert("game_state", new_game).await.unwrap();
        let game_id = fastrand::u64(1..10_000_000).to_string();
        session.insert("game_id", game_id.clone()).await.unwrap();
        Redirect::temporary(&format!("/{}/game/", game_id))
        //format!("The server is running well!\nYour game session has started! Use /game/guess to play.\nSession ID: {}", game_id);
    } else {// session_id in this case already exists
        //format!("The server is running well!\nSession already exists! Use /game/guess to continue playing.")
        let existing_id = session.get::<String>("game_id").await.unwrap();
        Redirect::temporary(&format!("/{}/game/", existing_id.unwrap()))
    }
}

#[tokio::main]
async fn main() {
    //GAME.lock().unwrap().restart();
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store)
    .with_secure(false)
    .with_expiry(OnInactivity(Duration::seconds(200)));

    let state =Arc::new(Mutex::new(GameApp {
        current_word: String::new(),
        right_guesses: vec![],
        wrong_guesses: vec![],
    }));
    
    // In the original book, the code was written with the deprecated Server struct
    // and the ServerBuilder struct. The ServerBuilder struct was removed in the last
    // version of axum.
    //
    // To serve the app, we use the `axum::serve` function which takes a TcpListener and an axum::Router.
    // In the given example, we use a Router to define different routes and their corresponding handlers.
    //
    // The use of `Arc<Mutex<GameApp>>` is a common way to share mutable state between multiple threads.
    //columns which differentiate variables in requests are not supported anymore in the recent releases of axum
    let app = axum::Router::new()
    .route("/", get(start_session))//replace with start session and this message
    .route("/{session_id}/game/{guess}", get(get_res_from_arc_mutex))
    .route("/{session_id}/game/", get(move || async { "The server runs well!"})) //This is added because we don't want the user to have
    // a 404 Http error when he is redirected to this url pattern
    .with_state(state)
    .layer(session_layer); //Adding the session middleware
    //Refactoring the code published in page 506 of the book due to the Server being deprecated and removed in 
    // the last release of axum
    // Explicitly parse the address as `SocketAddr`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // Create a TCP listener first
    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app)
    .await
    .unwrap();
}