# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_envio_global_optspecs
	string join \n h/help
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

complete -c envio -n "__fish_envio_needs_command" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_needs_command" -f -a "create" -d 'Create a new profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "add" -d 'Add envionment variables to a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "load" -d 'Load all environment variables in a profile for use in your terminal sessions'
complete -c envio -n "__fish_envio_needs_command" -f -a "unload" -d 'Unload a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "launch" -d 'Run a command with the environment variables from a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "remove" -d 'Remove a environment variable from a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "list" -d 'List all the environment variables in a profile or all the profiles currenty stored'
complete -c envio -n "__fish_envio_needs_command" -f -a "update" -d 'Update environment variables in a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "export" -d 'Export a profile to a file if no file is specified it will be exported to a file named .env'
complete -c envio -n "__fish_envio_needs_command" -f -a "import" -d 'Download a profile over the internet and import it into the system or import a locally stored profile into your current envio installation'
complete -c envio -n "__fish_envio_needs_command" -f -a "version" -d 'Print the version'
complete -c envio -n "__fish_envio_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c envio -n "__fish_envio_using_subcommand create" -s f -l file-to-import-envs-from -r
complete -c envio -n "__fish_envio_using_subcommand create" -s e -l envs -r
complete -c envio -n "__fish_envio_using_subcommand create" -s g -l gpg-key-fingerprint -r
complete -c envio -n "__fish_envio_using_subcommand create" -s n -l no-encryption
complete -c envio -n "__fish_envio_using_subcommand create" -s c -l add-comments
complete -c envio -n "__fish_envio_using_subcommand create" -s x -l add-expiration-date
complete -c envio -n "__fish_envio_using_subcommand create" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand add" -s e -l envs -r
complete -c envio -n "__fish_envio_using_subcommand add" -s c -l add-comments
complete -c envio -n "__fish_envio_using_subcommand add" -s x -l add-expiration-date
complete -c envio -n "__fish_envio_using_subcommand add" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand load" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand unload" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand launch" -s c -l command -r
complete -c envio -n "__fish_envio_using_subcommand launch" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand remove" -s e -l envs-to-remove -r
complete -c envio -n "__fish_envio_using_subcommand remove" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand list" -s n -l profile-name -r
complete -c envio -n "__fish_envio_using_subcommand list" -s p -l profiles
complete -c envio -n "__fish_envio_using_subcommand list" -s v -l no-pretty-print
complete -c envio -n "__fish_envio_using_subcommand list" -s c -l display-comments
complete -c envio -n "__fish_envio_using_subcommand list" -s x -l display-expiration-date
complete -c envio -n "__fish_envio_using_subcommand list" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand update" -s e -l envs -r
complete -c envio -n "__fish_envio_using_subcommand update" -s v -l update-values
complete -c envio -n "__fish_envio_using_subcommand update" -s c -l update-comments
complete -c envio -n "__fish_envio_using_subcommand update" -s x -l update-expiration-date
complete -c envio -n "__fish_envio_using_subcommand update" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand export" -s f -l file-to-export-to -r
complete -c envio -n "__fish_envio_using_subcommand export" -s e -l envs -r
complete -c envio -n "__fish_envio_using_subcommand export" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand import" -s f -l file-to-import-from -r
complete -c envio -n "__fish_envio_using_subcommand import" -s u -l url -r
complete -c envio -n "__fish_envio_using_subcommand import" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand version" -s v -l verbose
complete -c envio -n "__fish_envio_using_subcommand version" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "create" -d 'Create a new profile'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "add" -d 'Add envionment variables to a profile'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "load" -d 'Load all environment variables in a profile for use in your terminal sessions'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "unload" -d 'Unload a profile'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "launch" -d 'Run a command with the environment variables from a profile'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "remove" -d 'Remove a environment variable from a profile'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "list" -d 'List all the environment variables in a profile or all the profiles currenty stored'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "update" -d 'Update environment variables in a profile'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "export" -d 'Export a profile to a file if no file is specified it will be exported to a file named .env'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "import" -d 'Download a profile over the internet and import it into the system or import a locally stored profile into your current envio installation'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "version" -d 'Print the version'
complete -c envio -n "__fish_envio_using_subcommand help; and not __fish_seen_subcommand_from create add load unload launch remove list update export import version help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
