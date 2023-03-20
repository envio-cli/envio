// In this example we edit the environment variable of the profile passed as the first argument to the program

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        println!("Usage: <profile_name> <key> <env_var_to_edit> <new_value>");
        return;
    }

    let profile_name = args[1].to_string(); // The first argument is the name of the profile
    let key = args[2].to_string(); // The second argument is the key to decrypt the profile

    let mut profile = envio::get_profile(profile_name, key.clone()).unwrap();

    // print the environment variables in that profile before editing
    for (env_var, value) in &profile.envs {
        println!("{}: {}", env_var, value);
    }

    let env_var_to_edit = args[3].to_string(); // The second argument is the environment variable to edit
    let new_value = args[4].to_string(); // The third argument is the new value of the environment variable

    profile.edit_env(env_var_to_edit, new_value);
    profile.push_changes(key);

    // print the environment variables in that profile after editing
    for (env_var, value) in &profile.envs {
        println!("{}: {}", env_var, value);
    }
}
