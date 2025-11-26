# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_envio_global_optspecs
	string join \n diagnostic h/help
end

function __fish_envio_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_envio_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_envio_using_subcommand
	set -l cmd (__fish_envio_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c envio -n "__fish_envio_needs_command" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_needs_command" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_needs_command" -f -a "create" -d 'Create a new profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "new" -d 'Create a new profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "delete" -d 'Delete a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "remove" -d 'Delete a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "list" -d 'List all profiles'
complete -c envio -n "__fish_envio_needs_command" -f -a "ls" -d 'List all profiles'
complete -c envio -n "__fish_envio_needs_command" -f -a "show" -d 'Show environment variables in a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "set" -d 'Set or update environment variables in a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "unset" -d 'Remove environment variables from a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "load" -d 'Load environment variables from a profile for use in terminal sessions'
complete -c envio -n "__fish_envio_needs_command" -f -a "unload" -d 'Unload previously loaded environment variables from terminal sessions'
complete -c envio -n "__fish_envio_needs_command" -f -a "run" -d 'Run a command using environment variables from a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "exec" -d 'Run a command using environment variables from a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "import" -d 'Import a profile from a file or url'
complete -c envio -n "__fish_envio_needs_command" -f -a "export" -d 'Export the environment variables of a profile to a file'
complete -c envio -n "__fish_envio_needs_command" -f -a "tui" -d 'Launch the interactive TUI application'
complete -c envio -n "__fish_envio_needs_command" -f -a "completion" -d 'Show shell completion for the provided shell'
complete -c envio -n "__fish_envio_needs_command" -f -a "version" -d 'Print version information'
complete -c envio -n "__fish_envio_using_subcommand create" -s d -l description -d 'optional note or description of the profile' -r
complete -c envio -n "__fish_envio_using_subcommand create" -s f -l from-file -d 'file path to load environment variables from' -r
complete -c envio -n "__fish_envio_using_subcommand create" -s e -l envs -d 'environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)' -r
complete -c envio -n "__fish_envio_using_subcommand create" -s k -l cipher-kind -d 'encryption cipher to use' -r
complete -c envio -n "__fish_envio_using_subcommand create" -s c -l comments -d 'add comments to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand create" -s x -l expires -d 'add expiration dates to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand create" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand create" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand new" -s d -l description -d 'optional note or description of the profile' -r
complete -c envio -n "__fish_envio_using_subcommand new" -s f -l from-file -d 'file path to load environment variables from' -r
complete -c envio -n "__fish_envio_using_subcommand new" -s e -l envs -d 'environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)' -r
complete -c envio -n "__fish_envio_using_subcommand new" -s k -l cipher-kind -d 'encryption cipher to use' -r
complete -c envio -n "__fish_envio_using_subcommand new" -s c -l comments -d 'add comments to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand new" -s x -l expires -d 'add expiration dates to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand new" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand new" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand delete" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand delete" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand remove" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand remove" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand list" -l no-pretty-print -d 'disable pretty printing'
complete -c envio -n "__fish_envio_using_subcommand list" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand list" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand ls" -l no-pretty-print -d 'disable pretty printing'
complete -c envio -n "__fish_envio_using_subcommand ls" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand ls" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand show" -s c -l show-comments -d 'display comments'
complete -c envio -n "__fish_envio_using_subcommand show" -s x -l show-expiration -d 'display expiration dates'
complete -c envio -n "__fish_envio_using_subcommand show" -l no-pretty-print -d 'disable pretty printing'
complete -c envio -n "__fish_envio_using_subcommand show" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand show" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand set" -s c -l comments -d 'add comments to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand set" -s x -l expires -d 'add expiration dates to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand set" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand set" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand unset" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand unset" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand load" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand load" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand unload" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand unload" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand run" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand run" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand exec" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand exec" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand import" -s n -l profile-name -d 'name for the imported profile' -r
complete -c envio -n "__fish_envio_using_subcommand import" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand import" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand export" -s o -l output-file-path -d 'output file path (default: .env)' -r
complete -c envio -n "__fish_envio_using_subcommand export" -s k -l keys -d 'comma-separated list of keys to export (type \'select\' to choose interactively)' -r
complete -c envio -n "__fish_envio_using_subcommand export" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand export" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand tui" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand tui" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand completion" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand completion" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand version" -s v -l verbose -d 'show verbose version information'
complete -c envio -n "__fish_envio_using_subcommand version" -l diagnostic -d 'Show diagnostic information for bug reports'
complete -c envio -n "__fish_envio_using_subcommand version" -s h -l help -d 'Print help'
