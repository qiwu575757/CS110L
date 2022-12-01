// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect(); 
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)
    println!("Welcome to CS110L Hangman!\n");

    // vars used
    let mut guesses_left = 5;
    let mut guessed_letters = Vec::new();
    let mut word = Vec::new();

    // init the word 
    for i in 0..secret_word_chars.len() {
        word.push('-');
    }

    while guesses_left >= 1 {
        // print the word
        print!("The word so far is ");
        for i in 0..word.len() {
            print!("{}", word[i]);
        }
        println!("");

        // print the guessed letters
        print!("You have guessed the following letters: ");
        for i in 0..guessed_letters.len() {
            print!("{}", guessed_letters[i]);
        }
        println!("");

        println!("You have {} guesses left", guesses_left);

        print!("Please guess a letter: ");
        // Make sure the prompt from the previous line gets displayed:
        io::stdout()
            .flush()
            .expect("Error flushing stdout.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line.");

        // translate string into char vector
        let guess_char: Vec<char> = guess.chars().collect();
        let mut get_correct_char = false;
        for i in 0..secret_word_chars.len() {
            if secret_word_chars[i] == guess_char[0] && word[i] != guess_char[0] {
                word[i] = guess_char[0];
                guessed_letters.push(guess_char[0]);
                get_correct_char = true;
                break;
            } 
        }
        if !get_correct_char {
            println!("Sorry, that letter is not in the word");
            guesses_left -= 1;
        } 
        
        println!("");
        if word == secret_word_chars {
            break;
        }
    }

    if guesses_left <= 0 {
        println!("Sorry, you ran out of guesses!");
    } else {
        println!("Congratulations you guessed the secret word: {}!", secret_word);
    }

}
