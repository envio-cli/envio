# Project Structure

The `envio` CLI tool is organized into a directory structure that separates code into distinct modules.
Here is an overview of the project structure:

```sh
envio/
├── src/
│   ├── bin/
│   │   ├── cli.rs
│   │   ├── command.rs
│   │   └── main.rs
│   ├── lib.rs
│   ├── crypto.rs
│   └── utils.rs
```

The src directory contains all the code for `envio`. Here's what each file and directory does:

* `bin/`: This directory contains the executable code for the CLI tool.
    * `cli.rs`: This file defines the `Clap` application used to parse command-line arguments and invoke the appropriate subcommand.

    * `command.rs`: This file contains the implementation of each subcommand supported by `envio`.

    * `main.rs`: This file contains the entry point of the CLI tool and sets up other global configurations.

* `lib.rs`: This file contains the main logic of the `envio` library. It defines the API for loading, unloading, creating, and managing profiles and environments in a secure and efficient manner. The CLI tool uses this library.

* `crypto.rs`: This file contains functions and utilities related to cryptography, which is a key part of the `envio`.

* `utils.rs`: This file contains a collection of general-purpose utility functions used throughout the library and CLI tool.

Overall, this project structure makes it easy to navigate the codebase and understand how the different modules fit together. It also allows for easy extensibility, as new subcommands and library functions can be added as needed.