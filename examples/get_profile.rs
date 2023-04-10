// In this example we get the profile passed as an argument to the program and then print the environment variables in that profile

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: <profile_name> <key>");
        return;
    }

    let profile_name = args[1].to_string();
    let key = args[2].to_string(); // All profiles have a key that is used to encrypt the environment variables, this ensures that the environment variables are secure

    // We use the age encryption type here
    // If the profile was encrypted with a different encryption type you can use the encryption type that was used to encrypt the profile
    // For example if the profile was encrypted with the GPG encryption type you would use the following line instead:
    // let encryption_type = envio::crypto::create_encryption_type(key, "gpg"); -- Over here key would be the fingerprint of the GPG key used to encrypt the profile
    let encryption_type = envio::crypto::create_encryption_type(key, "age");

    // print the environment variables in that profile
    for (env_var, value) in &envio::get_profile(profile_name, encryption_type)
        .unwrap()
        .envs
    {
        println!("{}: {}", env_var, value);
    }
}
