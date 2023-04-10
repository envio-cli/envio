// In this example we edit the environment variable of the profile passed as the first argument to the program

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        println!("Usage: <profile_name> <key> <env_var_to_edit> <new_value>");
        return;
    }

    let profile_name = args[1].to_string(); // The first argument is the name of the profile
    let key = args[2].to_string(); // The second argument is the key to decrypt the profile

    // We use the age encryption type here
    // If the profile was encrypted with a different encryption type you can use the encryption type that was used to encrypt the profile
    // For example if the profile was encrypted with the GPG encryption type you would use the following line instead:
    // let encryption_type = envio::crypto::create_encryption_type(key, "gpg"); -- Over here key would be the fingerprint of the GPG key used to encrypt the profile
    let encryption_type = envio::crypto::create_encryption_type(key, "age");

    let mut profile = envio::get_profile(profile_name, encryption_type).unwrap();

    // print the environment variables in that profile before editing
    for (env_var, value) in &profile.envs {
        println!("{}: {}", env_var, value);
    }

    let env_var_to_edit = args[3].to_string(); // The second argument is the environment variable to edit
    let new_value = args[4].to_string(); // The third argument is the new value of the environment variable

    profile.edit_env(env_var_to_edit, new_value);
    profile.push_changes();

    // print the environment variables in that profile after editing
    for (env_var, value) in &profile.envs {
        println!("{}: {}", env_var, value);
    }
}
