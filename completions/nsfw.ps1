# PowerShell completion script for NSFW (Nix Subsystem for Windows)
#
# Installation:
#   1. Copy this file to: $PROFILE\..\Completions\nsfw.ps1
#   2. Add to your PowerShell profile ($PROFILE):
#      Import-Module "$PSScriptRoot\Completions\nsfw.ps1"
#
#   Or use the one-liner:
#   nsfw completion install
#
# Version: 0.2.0

# Register argument completer for nsfw command
Register-ArgumentCompleter -Native -CommandName nsfw -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @{
        'search' = 'Search for packages in nixpkgs'
        'install' = 'Install a package'
        'remove' = 'Remove an installed package'
        'list' = 'List installed packages'
        'info' = 'Show detailed package information'
        'update' = 'Update Nix channels'
        'setup' = 'Run first-time setup wizard'
        'generate-wrapper' = 'Generate Windows wrapper for a package'
        'help' = 'Show help information'
    }

    $globalOptions = @{
        '--help' = 'Print help information'
        '-h' = 'Print help information'
        '--version' = 'Print version information'
        '-V' = 'Print version information'
        '--verbose' = 'Enable verbose output'
        '-v' = 'Enable verbose output'
    }

    # Parse the command line to understand context
    $line = $commandAst.ToString()
    $words = $line -split '\s+' | Where-Object { $_ -ne '' }

    # Determine if we're completing a command or its arguments
    $currentCommand = $null
    if ($words.Count -gt 1) {
        $currentCommand = $words[1]
    }

    # If no command yet, suggest commands
    if (-not $currentCommand -or $words.Count -eq 1 -or ($words.Count -eq 2 -and $wordToComplete)) {
        $commands.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new(
                $_.Key,
                $_.Key,
                'ParameterValue',
                $_.Value
            )
        }

        # Also suggest global options
        $globalOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new(
                $_.Key,
                $_.Key,
                'ParameterName',
                $_.Value
            )
        }
        return
    }

    # Command-specific completions
    switch ($currentCommand) {
        'search' {
            $searchOptions = @{
                '--limit' = 'Maximum number of results (default: 20)'
                '-l' = 'Maximum number of results'
                '--format' = 'Output format: text, json'
                '-f' = 'Output format'
                '--help' = 'Show help for search command'
            }

            # If completing --format value
            if ($line -match '--format\s+$' -or $line -match '-f\s+$') {
                @('text', 'json') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "Format: $_")
                }
                return
            }

            $searchOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'install' {
            $installOptions = @{
                '--yes' = 'Skip confirmation prompt'
                '-y' = 'Skip confirmation prompt'
                '--help' = 'Show help for install command'
            }

            # Try to get cached package names for completion
            if ($wordToComplete -and -not $wordToComplete.StartsWith('-')) {
                # Get package suggestions from cache
                try {
                    $cacheFile = "$env:USERPROFILE\.nsfw\cache\packages.db"
                    if (Test-Path $cacheFile) {
                        # Query SQLite cache for package names (simplified - would need proper SQLite access)
                        # For now, suggest common packages
                        $commonPackages = @('python3', 'nodejs', 'git', 'vim', 'neovim', 'firefox',
                                          'chromium', 'vscode', 'rustc', 'go', 'gcc', 'cmake',
                                          'curl', 'wget', 'htop', 'tmux', 'zsh', 'fish')

                        $commonPackages | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "Package: $_")
                        }
                    }
                } catch {
                    # Silently fail if cache unavailable
                }
            }

            $installOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'remove' {
            $removeOptions = @{
                '--yes' = 'Skip confirmation prompt'
                '-y' = 'Skip confirmation prompt'
                '--help' = 'Show help for remove command'
            }

            # Suggest installed packages from 'nsfw list'
            if ($wordToComplete -and -not $wordToComplete.StartsWith('-')) {
                try {
                    $installedPackages = & nsfw list --format json 2>$null | ConvertFrom-Json
                    if ($installedPackages) {
                        $installedPackages | ForEach-Object {
                            if ($_.name -like "$wordToComplete*") {
                                [System.Management.Automation.CompletionResult]::new(
                                    $_.name,
                                    $_.name,
                                    'ParameterValue',
                                    "Remove: $($_.name) v$($_.version)"
                                )
                            }
                        }
                    }
                } catch {
                    # Silently fail if listing packages fails
                }
            }

            $removeOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'list' {
            $listOptions = @{
                '--detailed' = 'Show detailed package information'
                '-d' = 'Show detailed information'
                '--format' = 'Output format: text, json'
                '-f' = 'Output format'
                '--help' = 'Show help for list command'
            }

            if ($line -match '--format\s+$' -or $line -match '-f\s+$') {
                @('text', 'json') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "Format: $_")
                }
                return
            }

            $listOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'info' {
            $infoOptions = @{
                '--help' = 'Show help for info command'
            }

            # Suggest package names for info command
            if ($wordToComplete -and -not $wordToComplete.StartsWith('-')) {
                $commonPackages = @('python3', 'nodejs', 'git', 'vim', 'neovim', 'firefox',
                                  'chromium', 'vscode', 'rustc', 'go', 'gcc', 'cmake')

                $commonPackages | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "Get info: $_")
                }
            }

            $infoOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'update' {
            $updateOptions = @{
                '--help' = 'Show help for update command'
            }

            $updateOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'setup' {
            $setupOptions = @{
                '--yes' = 'Auto-confirm all prompts'
                '-y' = 'Auto-confirm all prompts'
                '--interactive' = 'Enable interactive mode'
                '-i' = 'Enable interactive mode'
                '--help' = 'Show help for setup command'
            }

            $setupOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'generate-wrapper' {
            $wrapperOptions = @{
                '--help' = 'Show help for generate-wrapper command'
            }

            # Suggest installed packages for wrapper generation
            if ($wordToComplete -and -not $wordToComplete.StartsWith('-')) {
                try {
                    $installedPackages = & nsfw list --format json 2>$null | ConvertFrom-Json
                    if ($installedPackages) {
                        $installedPackages | ForEach-Object {
                            if ($_.name -like "$wordToComplete*") {
                                [System.Management.Automation.CompletionResult]::new(
                                    $_.name,
                                    $_.name,
                                    'ParameterValue',
                                    "Generate wrapper for: $($_.name)"
                                )
                            }
                        }
                    }
                } catch {
                    # Silently fail
                }
            }

            $wrapperOptions.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterName', $_.Value)
            }
        }

        'help' {
            # Suggest commands for 'nsfw help <command>'
            $commands.GetEnumerator() | Where-Object { $_.Key -like "$wordToComplete*" -and $_.Key -ne 'help' } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_.Key, $_.Key, 'ParameterValue', $_.Value)
            }
        }
    }
}

# Helper function to install completions
function Install-NsfwCompletion {
    [CmdletBinding()]
    param()

    Write-Host "📦 Installing NSFW PowerShell Completions..." -ForegroundColor Cyan

    # Create completions directory if it doesn't exist
    $completionsDir = Join-Path (Split-Path $PROFILE) "Completions"
    if (-not (Test-Path $completionsDir)) {
        New-Item -ItemType Directory -Path $completionsDir -Force | Out-Null
        Write-Host "✓ Created completions directory: $completionsDir" -ForegroundColor Green
    }

    # Copy completion script
    $targetPath = Join-Path $completionsDir "nsfw.ps1"
    Copy-Item $PSCommandPath $targetPath -Force
    Write-Host "✓ Copied completion script to: $targetPath" -ForegroundColor Green

    # Check if already in profile
    $profileContent = Get-Content $PROFILE -ErrorAction SilentlyContinue
    $importLine = "Import-Module `"$targetPath`""

    if ($profileContent -notcontains $importLine) {
        Add-Content -Path $PROFILE -Value "`n# NSFW Tab Completions`n$importLine" -Force
        Write-Host "✓ Added import to PowerShell profile" -ForegroundColor Green
    } else {
        Write-Host "✓ Already configured in PowerShell profile" -ForegroundColor Yellow
    }

    Write-Host "`n🎉 Installation complete!" -ForegroundColor Green
    Write-Host "Please restart your PowerShell session or run:" -ForegroundColor Cyan
    Write-Host "  . `$PROFILE" -ForegroundColor White
}

# Export function
Export-ModuleMember -Function Install-NsfwCompletion

Write-Verbose "NSFW PowerShell completions loaded successfully (v0.2.0)"
