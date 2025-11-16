
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'envio' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'envio'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'envio' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('profile', 'profile', [CompletionResultType]::ParameterValue, 'profile')
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set or update environment variables in a profile')
            [CompletionResult]::new('unset', 'unset', [CompletionResultType]::ParameterValue, 'Remove an environment variable from a profile')
            [CompletionResult]::new('load', 'load', [CompletionResultType]::ParameterValue, 'Load all environment variables in a profile for use in your terminal sessions')
            [CompletionResult]::new('unload', 'unload', [CompletionResultType]::ParameterValue, 'Unload a profile')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Run a command with the environment variables from a profile')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import a profile from a file, URL, or .env file')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export a profile to a file')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print the version')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'envio;profile' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show environment variables in a profile')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'envio;profile;create' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--from-file', 'from-file', [CompletionResultType]::ParameterName, 'from-file')
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'e')
            [CompletionResult]::new('--envs', 'envs', [CompletionResultType]::ParameterName, 'envs')
            [CompletionResult]::new('-g', 'g', [CompletionResultType]::ParameterName, 'g')
            [CompletionResult]::new('--gpg-key', 'gpg-key', [CompletionResultType]::ParameterName, 'gpg-key')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--add-comments', 'add-comments', [CompletionResultType]::ParameterName, 'add-comments')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('--add-expiration-date', 'add-expiration-date', [CompletionResultType]::ParameterName, 'add-expiration-date')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;profile;delete' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;profile;list' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'v')
            [CompletionResult]::new('--no-pretty-print', 'no-pretty-print', [CompletionResultType]::ParameterName, 'no-pretty-print')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;profile;show' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'v')
            [CompletionResult]::new('--no-pretty-print', 'no-pretty-print', [CompletionResultType]::ParameterName, 'no-pretty-print')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--display-comments', 'display-comments', [CompletionResultType]::ParameterName, 'display-comments')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('--display-expiration-date', 'display-expiration-date', [CompletionResultType]::ParameterName, 'display-expiration-date')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;profile;help' {
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show environment variables in a profile')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'envio;profile;help;create' {
            break
        }
        'envio;profile;help;delete' {
            break
        }
        'envio;profile;help;list' {
            break
        }
        'envio;profile;help;show' {
            break
        }
        'envio;profile;help;help' {
            break
        }
        'envio;set' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--comments', 'comments', [CompletionResultType]::ParameterName, 'comments')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('--expiration-date', 'expiration-date', [CompletionResultType]::ParameterName, 'expiration-date')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;unset' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;load' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;unload' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;run' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;import' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'n')
            [CompletionResult]::new('--profile-name', 'profile-name', [CompletionResultType]::ParameterName, 'profile-name')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;export' {
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 't')
            [CompletionResult]::new('--to', 'to', [CompletionResultType]::ParameterName, 'to')
            [CompletionResult]::new('-k', 'k', [CompletionResultType]::ParameterName, 'k')
            [CompletionResult]::new('--keys', 'keys', [CompletionResultType]::ParameterName, 'keys')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;version' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'v')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'verbose')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;help' {
            [CompletionResult]::new('profile', 'profile', [CompletionResultType]::ParameterValue, 'profile')
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set or update environment variables in a profile')
            [CompletionResult]::new('unset', 'unset', [CompletionResultType]::ParameterValue, 'Remove an environment variable from a profile')
            [CompletionResult]::new('load', 'load', [CompletionResultType]::ParameterValue, 'Load all environment variables in a profile for use in your terminal sessions')
            [CompletionResult]::new('unload', 'unload', [CompletionResultType]::ParameterValue, 'Unload a profile')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Run a command with the environment variables from a profile')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import a profile from a file, URL, or .env file')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export a profile to a file')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print the version')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'envio;help;profile' {
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show environment variables in a profile')
            break
        }
        'envio;help;profile;create' {
            break
        }
        'envio;help;profile;delete' {
            break
        }
        'envio;help;profile;list' {
            break
        }
        'envio;help;profile;show' {
            break
        }
        'envio;help;set' {
            break
        }
        'envio;help;unset' {
            break
        }
        'envio;help;load' {
            break
        }
        'envio;help;unload' {
            break
        }
        'envio;help;run' {
            break
        }
        'envio;help;import' {
            break
        }
        'envio;help;export' {
            break
        }
        'envio;help;version' {
            break
        }
        'envio;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
