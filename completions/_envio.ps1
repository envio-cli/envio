
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
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('new', 'new', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a profile')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Delete a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show environment variables in a profile')
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set or update environment variables in a profile')
            [CompletionResult]::new('unset', 'unset', [CompletionResultType]::ParameterValue, 'Remove environment variables from a profile')
            [CompletionResult]::new('load', 'load', [CompletionResultType]::ParameterValue, 'Load environment variables from a profile for use in terminal sessions')
            [CompletionResult]::new('unload', 'unload', [CompletionResultType]::ParameterValue, 'Unload previously loaded environment variables from terminal sessions')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Run a command using environment variables from a profile')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import a profile from a file or url')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export the environment variables of a profile to a file')
            [CompletionResult]::new('tui', 'tui', [CompletionResultType]::ParameterValue, 'Launch the interactive TUI application')
            [CompletionResult]::new('completion', 'completion', [CompletionResultType]::ParameterValue, 'Show shell completion for the provided shell')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print version information')
            break
        }
        'envio;create' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'optional note or description of the profile')
            [CompletionResult]::new('--description', '--description', [CompletionResultType]::ParameterName, 'optional note or description of the profile')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'file path to load environment variables from')
            [CompletionResult]::new('--from-file', '--from-file', [CompletionResultType]::ParameterName, 'file path to load environment variables from')
            [CompletionResult]::new('-e', '-e', [CompletionResultType]::ParameterName, 'environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)')
            [CompletionResult]::new('--envs', '--envs', [CompletionResultType]::ParameterName, 'environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)')
            [CompletionResult]::new('-k', '-k', [CompletionResultType]::ParameterName, 'encryption cipher to use')
            [CompletionResult]::new('--cipher-kind', '--cipher-kind', [CompletionResultType]::ParameterName, 'encryption cipher to use')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'add comments to the provided environment variables')
            [CompletionResult]::new('--comments', '--comments', [CompletionResultType]::ParameterName, 'add comments to the provided environment variables')
            [CompletionResult]::new('-x', '-x', [CompletionResultType]::ParameterName, 'add expiration dates to the provided environment variables')
            [CompletionResult]::new('--expires', '--expires', [CompletionResultType]::ParameterName, 'add expiration dates to the provided environment variables')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;new' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'optional note or description of the profile')
            [CompletionResult]::new('--description', '--description', [CompletionResultType]::ParameterName, 'optional note or description of the profile')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'file path to load environment variables from')
            [CompletionResult]::new('--from-file', '--from-file', [CompletionResultType]::ParameterName, 'file path to load environment variables from')
            [CompletionResult]::new('-e', '-e', [CompletionResultType]::ParameterName, 'environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)')
            [CompletionResult]::new('--envs', '--envs', [CompletionResultType]::ParameterName, 'environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)')
            [CompletionResult]::new('-k', '-k', [CompletionResultType]::ParameterName, 'encryption cipher to use')
            [CompletionResult]::new('--cipher-kind', '--cipher-kind', [CompletionResultType]::ParameterName, 'encryption cipher to use')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'add comments to the provided environment variables')
            [CompletionResult]::new('--comments', '--comments', [CompletionResultType]::ParameterName, 'add comments to the provided environment variables')
            [CompletionResult]::new('-x', '-x', [CompletionResultType]::ParameterName, 'add expiration dates to the provided environment variables')
            [CompletionResult]::new('--expires', '--expires', [CompletionResultType]::ParameterName, 'add expiration dates to the provided environment variables')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;delete' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;remove' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;list' {
            [CompletionResult]::new('--no-pretty-print', '--no-pretty-print', [CompletionResultType]::ParameterName, 'disable pretty printing')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;ls' {
            [CompletionResult]::new('--no-pretty-print', '--no-pretty-print', [CompletionResultType]::ParameterName, 'disable pretty printing')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;show' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'display comments')
            [CompletionResult]::new('--show-comments', '--show-comments', [CompletionResultType]::ParameterName, 'display comments')
            [CompletionResult]::new('-x', '-x', [CompletionResultType]::ParameterName, 'display expiration dates')
            [CompletionResult]::new('--show-expiration', '--show-expiration', [CompletionResultType]::ParameterName, 'display expiration dates')
            [CompletionResult]::new('--no-pretty-print', '--no-pretty-print', [CompletionResultType]::ParameterName, 'disable pretty printing')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;set' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'add comments to the provided environment variables')
            [CompletionResult]::new('--comments', '--comments', [CompletionResultType]::ParameterName, 'add comments to the provided environment variables')
            [CompletionResult]::new('-x', '-x', [CompletionResultType]::ParameterName, 'add expiration dates to the provided environment variables')
            [CompletionResult]::new('--expires', '--expires', [CompletionResultType]::ParameterName, 'add expiration dates to the provided environment variables')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;unset' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;load' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;unload' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;run' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;import' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'name for the imported profile')
            [CompletionResult]::new('--profile-name', '--profile-name', [CompletionResultType]::ParameterName, 'name for the imported profile')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;export' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'output file path (default: .env)')
            [CompletionResult]::new('--output-file-path', '--output-file-path', [CompletionResultType]::ParameterName, 'output file path (default: .env)')
            [CompletionResult]::new('-k', '-k', [CompletionResultType]::ParameterName, 'comma-separated list of keys to export (type ''select'' to choose interactively)')
            [CompletionResult]::new('--keys', '--keys', [CompletionResultType]::ParameterName, 'comma-separated list of keys to export (type ''select'' to choose interactively)')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;tui' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;completion' {
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;version' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'show verbose version information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'show verbose version information')
            [CompletionResult]::new('--diagnostic', '--diagnostic', [CompletionResultType]::ParameterName, 'Show diagnostic information for bug reports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
