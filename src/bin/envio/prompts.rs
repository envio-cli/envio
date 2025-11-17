use std::{env, path::Path};

use envio::error::{Error, Result};
use inquire::{
    min_length, Confirm, DateSelect, MultiSelect, Password, PasswordDisplayMode, Select, Text,
};
use regex::Regex;

pub struct PasswordPromptOptions {
    pub title: String,
    pub help_message: Option<String>,
    pub min_length: Option<usize>,
    pub with_confirmation: bool,
    pub confirmation_error_message: Option<String>,
}

pub fn password_prompt(options: PasswordPromptOptions) -> Result<String> {
    let mut prompt = Password::new(&options.title)
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_display_toggle_enabled()
        .with_help_message(options.help_message.as_deref().unwrap_or(""))
        .with_validator(min_length!(options.min_length.unwrap_or(8)));

    if options.with_confirmation {
        prompt = prompt.with_custom_confirmation_error_message(
            options
                .confirmation_error_message
                .as_deref()
                .unwrap_or("The passwords don't match"),
        );
    } else {
        prompt = prompt.without_confirmation();
    }

    prompt.prompt().map_err(|e| Error::Msg(e.to_string()))
}

pub struct SelectPromptOptions<T> {
    pub title: String,
    pub options: Vec<T>,
}

pub fn select_prompt<T>(options: SelectPromptOptions<T>) -> Result<T>
where
    T: std::fmt::Display,
{
    Select::new(&options.title, options.options)
        .with_vim_mode(get_vim_mode()?)
        .with_help_message("↑↓ to move, space to select, type to filter, enter to confirm")
        .prompt()
        .map_err(|e| Error::Msg(e.to_string()))
}

pub struct TextPromptOptions {
    pub title: String,
    pub default: Option<String>,
}

pub fn text_prompt(options: TextPromptOptions) -> Result<String> {
    Text::new(&options.title)
        .with_default(options.default.as_deref().unwrap_or(""))
        .prompt()
        .map_err(|e| Error::Msg(e.to_string()))
}

pub struct ConfirmPromptOptions {
    pub title: String,
    pub help_message: Option<String>,
    pub default: bool,
}

pub fn confirm_prompt(options: ConfirmPromptOptions) -> Result<bool> {
    Confirm::new(&options.title)
        .with_default(options.default)
        .with_help_message(options.help_message.as_deref().unwrap_or(""))
        .prompt()
        .map_err(|e| Error::Msg(e.to_string()))
}

pub struct DatePromptOptions {
    pub title: String,
    pub default: Option<chrono::NaiveDate>,
}

pub fn date_prompt(options: DatePromptOptions) -> Result<chrono::NaiveDate> {
    DateSelect::new(&options.title)
        .with_default(options.default.unwrap_or(chrono::Local::now().date_naive()))
        .prompt()
        .map_err(|e| Error::Msg(e.to_string()))
}

pub struct MultiSelectPromptOptions<T> {
    pub title: String,
    pub options: Vec<T>,
    pub default_indices: Option<Vec<usize>>,
}

pub fn multi_select_prompt<T>(options: MultiSelectPromptOptions<T>) -> Result<Vec<T>>
where
    T: std::fmt::Display,
{
    MultiSelect::new(&options.title, options.options)
        .with_default(&options.default_indices.unwrap_or_default())
        .with_vim_mode(get_vim_mode()?)
        .with_help_message("↑↓ to move, space to select/unselect one, → to all, ← to none, type to filter, enter to confirm")
        .prompt()
        .map_err(|e| Error::Msg(e.to_string()))
}

fn get_vim_mode() -> Result<bool> {
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_default();

    if let Some(program) = editor.split_whitespace().next() {
        if let Some(stem) = Path::new(program).file_stem().and_then(|s| s.to_str()) {
            return Ok(Regex::new(r"n?vim?").unwrap().is_match(stem));
        }
    }

    Ok(false)
}
