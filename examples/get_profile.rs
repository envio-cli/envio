// In this example we get the profile passed as an argument to the program and then print the environment variables in that profile

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: <profile_name> <key>");
        return;
    }

    let profile_name = args[1].to_string();
    let key = args[2].to_string(); // All profiles have a key that is used to encrypt the environment variables, this ensures that the environment variables are secure

    // print the environment variables in that profile
    for (env_var, value) in &envio::get_profile(profile_name, key).unwrap().envs {
        println!("{}: {}", env_var, value);
    }
}
