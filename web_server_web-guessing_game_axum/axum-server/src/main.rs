use axum::{extract::{Path, State}, routing::get};
//use tokio::net::TcpListener;
use std::{/*net::SocketAddr,*/sync::{Mutex,Arc}};
use fastrand;
use shuttle_axum::ShuttleAxum;

const RANDOM_WORDS: [&str; 6] =["MB", "Windy", "Gomes", "Johnny", "Seoul", "Interesting"];

#[derive(Clone, Debug)]
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

async fn get_res_from_arc_mutex(State(game): State<Arc<Mutex<GameApp>>>,Path(guess): Path<String>) -> String {
    let mut game=game.lock().unwrap();
    game.take_guess(guess)
    }

impl GameApp {
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
                output.push_str(&format!("You guessed right, it's {}! Let's play again!", self.current_word));
                } else {
                output.push_str(&format!( "Bzzt! It's not {guess}, it's {}.\nTime to move on to another word!", self.current_word));
                }
                self.restart();
            }
        }
    output
    }
}

//#[tokio::main]
#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    let state =Arc::new(Mutex::new(GameApp {
        current_word: String::new(),
        right_guesses: vec![],
        wrong_guesses: vec![],
    }));
    
    let app = axum::Router::new()
    .route("/", get(|| async { "The server is running well!" }))
    .route("/game/{guess}", get(get_res_from_arc_mutex))
    .with_state(state);
    //let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    //let listener = TcpListener::bind(addr).await.unwrap();

    //axum::serve(listener, app).await.unwrap();
    Ok(app.into())
}

// I got this link from shuttle which you can try to test the deployment: https://axum-server-zu5z.shuttle.app