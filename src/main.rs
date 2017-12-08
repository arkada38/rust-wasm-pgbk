#![feature(string_retain)]

use std::mem;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc_str(ptr: *mut c_char) {
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

const UPPERCASE_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE_LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "1234567890";
const SPECIAL_CHARACTERS: &str = "!#$%'()*+-./:?@[]^_`{}~";

const M: usize = (38 * 4 + 3) * (62 * 4 + 3); // p % 4 = 3 // q % 4 = 3 // m = p * q
static mut SEED: usize = 0;

#[no_mangle]
pub fn generate_password(
    service_name_c: *mut c_char,
    keyword_c: *mut c_char,
    password_length: i32,
    use_special_characters: bool) -> *mut c_char {
    let service_name: String;
    let keyword: String;

    unsafe {
        service_name = CString::from_raw(service_name_c).into_string().unwrap();
        keyword = CString::from_raw(keyword_c).into_string().unwrap();
    }
    
    let service_name_score = get_score_of_utf8_bytes(&service_name);
    let keyword_score = get_score_of_utf8_bytes(&keyword);

    unsafe {
        SEED = (service_name_score + service_name_score % 9 +
            keyword_score - keyword_score % 4 -
            password_length + password_length % 2) as usize;
    }

    //Password should be contain uppercase, lowercase, numbers and special characters
    //without repeated characters
    //and consecutive of letters or numbers
    let mut quantity_of_special_characters = 0;
    if use_special_characters {
        quantity_of_special_characters = i32::max(2, password_length / 5);
    }
    let quantity_of_numbers = i32::min(NUMBERS.chars().count() as i32, password_length / 4);
    let quantity_of_uppercase_letters = (password_length - quantity_of_numbers - quantity_of_special_characters) / 2;
    let quantity_of_lowercase_letters = password_length - quantity_of_numbers - quantity_of_special_characters - quantity_of_uppercase_letters;

    let mut password_of_numbers = String::new();
    let mut password_of_special_characters = String::new();
    let mut password_of_uppercase_letters = String::new();
    let mut password_of_lowercase_letters = String::new();

    //quantity_of_special_characters <= quantity_of_numbers <= quantity_of_uppercase_letters <= quantity_of_lowercase_letters
    
    //uppercase
    let mut uppercase_letters = String::from(UPPERCASE_LETTERS);
    for _ in 0..quantity_of_uppercase_letters {
        let j = next_random(uppercase_letters.chars().count());
        password_of_uppercase_letters.push(uppercase_letters.remove(j));
    }

    unsafe {
        SEED += service_name.chars().count();
    }

    //lowercase
    let mut lowercase_letters = String::from(LOWERCASE_LETTERS);
    for _ in 0..quantity_of_lowercase_letters {
        let j = next_random(lowercase_letters.chars().count());
        password_of_lowercase_letters.push(lowercase_letters.remove(j));
    }

    unsafe {
        SEED += keyword.chars().count();
    }

    //numbers
    let mut numbers = String::from(NUMBERS);
    for _ in 0..quantity_of_numbers {
        let j = next_random(numbers.chars().count());
        password_of_numbers.push(numbers.remove(j));
    }

    unsafe {
        SEED += password_length as usize;
    }

    //specialCharacters
    let mut special_characters = String::from(SPECIAL_CHARACTERS);
    for _ in 0..quantity_of_special_characters {
        let j = next_random(special_characters.chars().count());
        password_of_special_characters.push(special_characters.remove(j));
    }

    unsafe {
        SEED += service_name.chars().count() + keyword.chars().count() + password_length as usize;
    }

    //Creation a password
    #[derive(PartialEq)]
    enum SymbolFrom {
        Numbers,
        SpecialCharacters,
        UppercaseLetters,
        LowercaseLetters,
    }
    let mut password = String::new();
    let mut last_symbol_for_password_from: Option<SymbolFrom> = None;
    let mut max_quantity: usize;

    for _ in 0..password_length {
        //The maximum value of characters in the sequence category for the password
        max_quantity = usize::max(usize::max(password_of_numbers.chars().count(), password_of_special_characters.chars().count()),
            usize::max(password_of_uppercase_letters.chars().count(), password_of_lowercase_letters.chars().count()));

        //The new password symbol will be from one of the longest sequences
        let mut new_symbol = String::new();

        //Select applicants
        if password_of_numbers.chars().count() == max_quantity && last_symbol_for_password_from != Some(SymbolFrom::Numbers) && password.chars().count() > 0 {
            new_symbol.push(password_of_numbers.chars().nth(0).unwrap());
        }

        if password_of_special_characters.chars().count() == max_quantity && last_symbol_for_password_from != Some(SymbolFrom::SpecialCharacters) && password.chars().count() > 0 {
            new_symbol.push(password_of_special_characters.chars().nth(0).unwrap());
        }

        if password_of_uppercase_letters.chars().count() == max_quantity && last_symbol_for_password_from != Some(SymbolFrom::UppercaseLetters) {
            new_symbol.push(password_of_uppercase_letters.chars().nth(0).unwrap());
        }

        if password_of_lowercase_letters.chars().count() == max_quantity && last_symbol_for_password_from != Some(SymbolFrom::LowercaseLetters) {
            new_symbol.push(password_of_lowercase_letters.chars().nth(0).unwrap());
        }

        //That symbol
        let nth = next_random(new_symbol.chars().count());
        let new_symbol = new_symbol.remove(nth);
        unsafe {
            SEED += 1;
        }

        //Remove the character from the sequence and remember the sequence
        if password_of_numbers.contains(new_symbol)
        {
            password_of_numbers.retain(|c| c != new_symbol);
            last_symbol_for_password_from = Some(SymbolFrom::Numbers);
        }
        else if password_of_special_characters.contains(new_symbol)
        {
            password_of_special_characters.retain(|c| c != new_symbol);
            last_symbol_for_password_from = Some(SymbolFrom::SpecialCharacters);
        }
        else if password_of_uppercase_letters.contains(new_symbol)
        {
            password_of_uppercase_letters.retain(|c| c != new_symbol);
            last_symbol_for_password_from = Some(SymbolFrom::UppercaseLetters);
        }
        else if password_of_lowercase_letters.contains(new_symbol)
        {
            password_of_lowercase_letters.retain(|c| c != new_symbol);
            last_symbol_for_password_from = Some(SymbolFrom::LowercaseLetters);
        }

        //Add to the password
        password.push(new_symbol);
    }
    

    CString::new(password.to_string()).unwrap().into_raw()
}

fn get_score_of_utf8_bytes(s: &String) -> i32 {
    let mut score = 0;

    for (i, c) in s.as_bytes().iter().enumerate() {
        if i % 2 != 0 {
            score += 2 * *c as i32
        } else {
            score -= *c as i32
        }
    }

    score % 1000
}

fn next_random(n: usize) -> usize {
    let x: f64;

    unsafe {
        SEED = SEED * SEED % M;
        x = ((n - 1) * SEED) as f64 / M as f64;
    }
    
    x.round() as usize
}
