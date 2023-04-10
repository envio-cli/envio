// In this example we download the profile passed as an argument to the program and then save it

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        println!("Usage: <url> <profile_name> <key>");
        return;
    }

    let url = args[1].to_string(); // The first argument is the url of the profile filehe second argument is the key used to encrypt the profile file
    let profile_name = args[2].to_string(); // The third argument is the name that the profile will be saved as

    envio::download_profile(url, profile_name.clone());

    // Check that the profile was downloaded correctly
    // Make sure you have the key that was used to encrypt the profile file or else you won't be able to decrypt it
    let key = args[3].to_string();

    // We use the age encryption type here
    // If the profile was encrypted with a different encryption type you can use the encryption type that was used to encrypt the profile
    // For example if the profile was encrypted with the GPG encryption type you would use the following line instead:
    // let encryption_type = envio::crypto::create_encryption_type(key, "gpg"); -- Over here key would be the fingerprint of the GPG key used to encrypt the profile
    let encryption_type = envio::crypto::create_encryption_type(key, "age");

    for (env_var, value) in &envio::get_profile(profile_name, encryption_type)
        .unwrap()
        .envs
    {
        println!("{}: {}", env_var, value);
    }
}
