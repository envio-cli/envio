// In this example we export the profile passed as an argument to the program to a file passed as the second argument

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        println!("Usage: <profile_name> <file_to_save> <key>");
        return;
    }

    let profile_name = args[1].to_string(); // The first argument is the name of the profile to export
    let file_to_save = args[2].to_string(); // The second argument is the path to the file to export the profile to
    let key = args[3].to_string(); // The third argument is the key to decrypt the profile

    // exporting the environment variables in the profile
    let profile = envio::get_profile(profile_name, key);
    profile.unwrap().export_envs(file_to_save);
}
