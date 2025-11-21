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
complete -c envio -n "__fish_envio_needs_command" -f -a "profile" -d 'manage profiles'
complete -c envio -n "__fish_envio_needs_command" -f -a "set" -d 'set or update environment variables in a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "unset" -d 'remove environment variables from a profile'
complete -c envio -n "__fish_envio_needs_command" -f -a "load" -d 'load environment variables from a profile for use in the current terminal session'
complete -c envio -n "__fish_envio_needs_command" -f -a "unload" -d 'unload a profile from the current terminal session'
complete -c envio -n "__fish_envio_needs_command" -f -a "run" -d 'run a command with profile environment variables'
complete -c envio -n "__fish_envio_needs_command" -f -a "import" -d 'import a profile from a file or url'
complete -c envio -n "__fish_envio_needs_command" -f -a "export" -d 'export the environment variables of a profile to a file'
complete -c envio -n "__fish_envio_needs_command" -f -a "version" -d 'print version information'
complete -c envio -n "__fish_envio_using_subcommand profile; and not __fish_seen_subcommand_from create delete list show" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand profile; and not __fish_seen_subcommand_from create delete list show" -f -a "create" -d 'create a new profile'
complete -c envio -n "__fish_envio_using_subcommand profile; and not __fish_seen_subcommand_from create delete list show" -f -a "delete" -d 'delete a profile'
complete -c envio -n "__fish_envio_using_subcommand profile; and not __fish_seen_subcommand_from create delete list show" -f -a "list" -d 'list all profiles'
complete -c envio -n "__fish_envio_using_subcommand profile; and not __fish_seen_subcommand_from create delete list show" -f -a "show" -d 'show environment variables in a profile'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from create" -s d -l description -d 'optional note or description of the profile' -r
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from create" -s f -l from-file -d 'file path to load environment variables from' -r
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from create" -s e -l envs -d 'environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)' -r
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from create" -s k -l cipher-kind -d 'encryption cipher to use' -r
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from create" -s c -l comments -d 'add comments to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from create" -s x -l expires -d 'add expiration dates to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from list" -l no-pretty-print -d 'disable pretty printing'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from show" -s c -l show-comments -d 'display comments'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from show" -s x -l show-expiration -d 'display expiration dates'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from show" -l no-pretty-print -d 'disable pretty printing'
complete -c envio -n "__fish_envio_using_subcommand profile; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand set" -s c -l comments -d 'add comments to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand set" -s x -l expires -d 'add expiration dates to the provided environment variables'
complete -c envio -n "__fish_envio_using_subcommand set" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand unset" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand load" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand unload" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand run" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand import" -s n -l profile-name -d 'name for the imported profile' -r
complete -c envio -n "__fish_envio_using_subcommand import" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand export" -s o -l output-file-path -d 'output file path (default: .env)' -r
complete -c envio -n "__fish_envio_using_subcommand export" -s k -l keys -d 'comma-separated list of keys to export (type \'select\' to choose interactively)' -r
complete -c envio -n "__fish_envio_using_subcommand export" -s h -l help -d 'Print help'
complete -c envio -n "__fish_envio_using_subcommand version" -s v -l verbose -d 'show verbose version information'
complete -c envio -n "__fish_envio_using_subcommand version" -s h -l help -d 'Print help'
