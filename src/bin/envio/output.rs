use colored::Colorize;

pub fn success<T>(message: T)
where
    T: std::fmt::Display,
{
    println!("{}: {}", "Success".green(), message);
}

pub fn error<T>(message: T)
where
    T: std::fmt::Display,
{
    eprintln!("{}: {}", "Error".red(), message);
}

pub fn warning<T>(message: T)
where
    T: std::fmt::Display,
{
    println!("{}: {}", "Warning".yellow(), message);
}
