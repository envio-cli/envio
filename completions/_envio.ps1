
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
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new environment variable to a profile')
            [CompletionResult]::new('load', 'load', [CompletionResultType]::ParameterValue, 'Load a profile in the current session')
            [CompletionResult]::new('unload', 'unload', [CompletionResultType]::ParameterValue, 'Unload a profile from the current session')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a environment variable from a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all the environment variables in a profile or all the profiles')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update a environment variable in a profile')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export a profile to a file if no file is specified it will be exported to a file named .env')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import a profile from a file')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print the version')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'envio;create' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;add' {
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
        'envio;remove' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;list' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;update' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;export' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;import' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;version' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'envio;help' {
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new profile')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new environment variable to a profile')
            [CompletionResult]::new('load', 'load', [CompletionResultType]::ParameterValue, 'Load a profile in the current session')
            [CompletionResult]::new('unload', 'unload', [CompletionResultType]::ParameterValue, 'Unload a profile from the current session')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a environment variable from a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all the environment variables in a profile or all the profiles')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update a environment variable in a profile')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export a profile to a file if no file is specified it will be exported to a file named .env')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import a profile from a file')
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
