# Important Security Notice for Envio Users

### Security Vulnerability in Versions Before 0.4.0

A security vulnerability has recently been discovered in versions prior to `0.4.0`. Before version `0.4.0`, users could load their profile using the `envio load <profile_name>` command and load the environment variables from their profile persistently. However, the way this was achieved was risky and insecure.

Specifically, the `envio load` command would take in the user key, decrypt the contents in the profile, take the environment variables, and then write them to the `setenv.sh` file, which would be sourced from the user's shell. Since the environment variables were written in plain text in `setenv.sh`, it became vulnerable to potential security breaches.

While it was not completely unsecure because users still had to pass in their key before loading in the environment variables from their profile and only then would the environment variables get written to and exported from the `setenv.sh` script, Nevertheless it was not a good approach and is not recommended.

### Updated Approach in Versions After 0.4.0

Now, with the update in version `0.4.0`, `envio` has implemented an updated approach. Whenever users use the `envio load` command, it creates a `setenv.sh` script (as before) that, whenever the users load their shell, asks for the user's key. If the key is correct, it decrypts the envs in the profile, writes them to a temporary file, sources the temporary file, and then deletes it. This approach ensures that the user's environment variables remain secure and are not exposed in plain text.

With this new approach, users can still load their profiles as before using the `envio load` command, but now whenever they open their shell, they need to enter their key to access their environment variables.

Users can also still use the `envio unload` command to unload the profile from their terminal sessions, but they do not need to pass the `profile name` as a argument anymore

### Future Improvements in Envio

`envio` is committed to continuously improving the way it loads environment variables until it reaches a certain level of satisfaction. Until version 1.0.0, `envio` will keep working on improving the way it loads environment variables, and users can expect changes in the approach. However, after version 1.0.0, `envio` will stabilize the approach, and users can expect fewer changes that won't be breaking.

We strongly encourage all users to upgrade to version 0.4.0 and take necessary measures to ensure their environment variables are secure.

### User Feedback

We would love to hear your thoughts and feedback on the new approach we use to load environment variables. If you have any suggestions or improvements, please don't hesitate to reach out to us at `humblepenguinofficial@gmail.com`
