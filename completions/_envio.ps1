
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
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add envionment variables to a profile')
            [CompletionResult]::new('load', 'load', [CompletionResultType]::ParameterValue, 'Load all environment variables in a profile for use in your terminal sessions')
            [CompletionResult]::new('unload', 'unload', [CompletionResultType]::ParameterValue, 'Unload a profile')
            [CompletionResult]::new('launch', 'launch', [CompletionResultType]::ParameterValue, 'Run a command with the environment variables from a profile')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a environment variable from a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all the environment variables in a profile or all the profiles currenty stored')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update environment variables in a profile')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export a profile to a file if no file is specified it will be exported to a file named .env')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Download a profile over the internet and import it into the system or import a locally stored profile into your current envio installation')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print the version')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'envio;create' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--file-to-import-envs-from', 'file-to-import-envs-from', [CompletionResultType]::ParameterName, 'file-to-import-envs-from')
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'e')
            [CompletionResult]::new('--envs', 'envs', [CompletionResultType]::ParameterName, 'envs')
            [CompletionResult]::new('-g', 'g', [CompletionResultType]::ParameterName, 'g')
            [CompletionResult]::new('--gpg-key-fingerprint', 'gpg-key-fingerprint', [CompletionResultType]::ParameterName, 'gpg-key-fingerprint')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--add-comments', 'add-comments', [CompletionResultType]::ParameterName, 'add-comments')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('--add-expiration-date', 'add-expiration-date', [CompletionResultType]::ParameterName, 'add-expiration-date')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;add' {
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'e')
            [CompletionResult]::new('--envs', 'envs', [CompletionResultType]::ParameterName, 'envs')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--add-comments', 'add-comments', [CompletionResultType]::ParameterName, 'add-comments')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('--add-expiration-date', 'add-expiration-date', [CompletionResultType]::ParameterName, 'add-expiration-date')
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
        'envio;launch' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--command', 'command', [CompletionResultType]::ParameterName, 'command')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;remove' {
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'e')
            [CompletionResult]::new('--envs-to-remove', 'envs-to-remove', [CompletionResultType]::ParameterName, 'envs-to-remove')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;list' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'n')
            [CompletionResult]::new('--profile-name', 'profile-name', [CompletionResultType]::ParameterName, 'profile-name')
            [CompletionResult]::new('-p', 'p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--profiles', 'profiles', [CompletionResultType]::ParameterName, 'profiles')
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
        'envio;update' {
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'e')
            [CompletionResult]::new('--envs', 'envs', [CompletionResultType]::ParameterName, 'envs')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'v')
            [CompletionResult]::new('--update-values', 'update-values', [CompletionResultType]::ParameterName, 'update-values')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--update-comments', 'update-comments', [CompletionResultType]::ParameterName, 'update-comments')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('--update-expiration-date', 'update-expiration-date', [CompletionResultType]::ParameterName, 'update-expiration-date')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;export' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--file-to-export-to', 'file-to-export-to', [CompletionResultType]::ParameterName, 'file-to-export-to')
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'e')
            [CompletionResult]::new('--envs', 'envs', [CompletionResultType]::ParameterName, 'envs')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;import' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--file-to-import-from', 'file-to-import-from', [CompletionResultType]::ParameterName, 'file-to-import-from')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'u')
            [CompletionResult]::new('--url', 'url', [CompletionResultType]::ParameterName, 'url')
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
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add envionment variables to a profile')
            [CompletionResult]::new('load', 'load', [CompletionResultType]::ParameterValue, 'Load all environment variables in a profile for use in your terminal sessions')
            [CompletionResult]::new('unload', 'unload', [CompletionResultType]::ParameterValue, 'Unload a profile')
            [CompletionResult]::new('launch', 'launch', [CompletionResultType]::ParameterValue, 'Run a command with the environment variables from a profile')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a environment variable from a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all the environment variables in a profile or all the profiles currenty stored')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update environment variables in a profile')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export a profile to a file if no file is specified it will be exported to a file named .env')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Download a profile over the internet and import it into the system or import a locally stored profile into your current envio installation')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print the version')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'envio;help;create' {
            break
        }
        'envio;help;add' {
            break
        }
        'envio;help;load' {
            break
        }
        'envio;help;unload' {
            break
        }
        'envio;help;launch' {
            break
        }
        'envio;help;remove' {
            break
        }
        'envio;help;list' {
            break
        }
        'envio;help;update' {
            break
        }
        'envio;help;export' {
            break
        }
        'envio;help;import' {
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
