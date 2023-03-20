// In this example we create a new profile with the name passed as the first argument to the program and then print the environment variables in that profile

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: <profile_name> <key>");
        return;
    }

    let profile_name = args[1].to_string(); // The first argument is the name of the profile to create
    let key = args[2].to_string(); // The second argument is the key to encrypt the profile (this is the same key that will be used to decrypt the profile so make sure you remember it)

    let envs = std::collections::HashMap::new(); // The environment variables to add to the profile

    envio::create_profile(profile_name, Some(envs), key);
}
