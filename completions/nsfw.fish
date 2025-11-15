# Fish completion script for NSFW (Nix Subsystem for Windows)
# Install: Copy to ~/.config/fish/completions/nsfw.fish

# Remove any existing completions
complete -c nsfw -e

# Global options
complete -c nsfw -s h -l help -d 'Show help information'
complete -c nsfw -s v -l verbose -d 'Enable verbose logging'
complete -c nsfw -l version -d 'Show version information'

# Main commands
complete -c nsfw -f -n '__fish_use_subcommand' -a 'search' -d 'Search for packages in nixpkgs'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'find' -d 'Alias for search'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'install' -d 'Install a package'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'add' -d 'Alias for install'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'remove' -d 'Remove an installed package'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'uninstall' -d 'Alias for remove'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'list' -d 'List installed packages'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'ls' -d 'Alias for list'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'info' -d 'Show information about a package'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'update' -d 'Update the package database'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'setup' -d 'Setup WSL2 and Nix environment'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'cache' -d 'Manage package cache'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'doctor' -d 'Diagnose system health'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'completion' -d 'Install shell completions'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'config' -d 'Manage configuration settings'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'upgrade' -d 'Upgrade installed packages'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'export' -d 'Export installed packages'
complete -c nsfw -f -n '__fish_use_subcommand' -a 'import' -d 'Import packages from file'

# search/find command
complete -c nsfw -f -n '__fish_seen_subcommand_from search find' -s l -l limit -d 'Maximum number of results' -r
complete -c nsfw -f -n '__fish_seen_subcommand_from search find' -s f -l format -d 'Output format' -a 'text json'
complete -c nsfw -f -n '__fish_seen_subcommand_from search find' -s h -l help -d 'Show help'

# install/add command
complete -c nsfw -f -n '__fish_seen_subcommand_from install add' -s y -l yes -d 'Skip confirmation prompt'
complete -c nsfw -f -n '__fish_seen_subcommand_from install add' -s h -l help -d 'Show help'

# remove/uninstall command
complete -c nsfw -f -n '__fish_seen_subcommand_from remove uninstall' -s y -l yes -d 'Skip confirmation prompt'
complete -c nsfw -f -n '__fish_seen_subcommand_from remove uninstall' -s h -l help -d 'Show help'

# list/ls command
complete -c nsfw -f -n '__fish_seen_subcommand_from list ls' -s d -l detailed -d 'Show detailed information'
complete -c nsfw -f -n '__fish_seen_subcommand_from list ls' -s f -l format -d 'Output format' -a 'text json'
complete -c nsfw -f -n '__fish_seen_subcommand_from list ls' -s h -l help -d 'Show help'

# info command
complete -c nsfw -f -n '__fish_seen_subcommand_from info' -s h -l help -d 'Show help'

# update command
complete -c nsfw -f -n '__fish_seen_subcommand_from update' -s h -l help -d 'Show help'

# setup command
complete -c nsfw -f -n '__fish_seen_subcommand_from setup' -s y -l yes -d 'Skip confirmation prompts'
complete -c nsfw -f -n '__fish_seen_subcommand_from setup' -s i -l interactive -d 'Interactive mode'
complete -c nsfw -f -n '__fish_seen_subcommand_from setup' -s h -l help -d 'Show help'

# cache command
complete -c nsfw -f -n '__fish_seen_subcommand_from cache; and not __fish_seen_subcommand_from stats clear rebuild' -a 'stats' -d 'Show cache statistics'
complete -c nsfw -f -n '__fish_seen_subcommand_from cache; and not __fish_seen_subcommand_from stats clear rebuild' -a 'clear' -d 'Clear the package cache'
complete -c nsfw -f -n '__fish_seen_subcommand_from cache; and not __fish_seen_subcommand_from stats clear rebuild' -a 'rebuild' -d 'Rebuild the package cache'

# config command
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show get set reset path keys' -a 'show' -d 'Show all configuration settings'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show get set reset path keys' -a 'get' -d 'Get a specific configuration value'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show get set reset path keys' -a 'set' -d 'Set a configuration value'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show get set reset path keys' -a 'reset' -d 'Reset to defaults'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show get set reset path keys' -a 'path' -d 'Show config file path'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show get set reset path keys' -a 'keys' -d 'List all config keys'

# config get/set keys
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'cache_ttl_days' -d 'Days before cache expires'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'default_wrapper_type' -d 'Default wrapper type'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'auto_update_channels' -d 'Auto-update before operations'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'install_location' -d 'Custom install path'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'verbose_output' -d 'Enable verbose logging'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'disable_colors' -d 'Disable colored output'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'parallel_jobs' -d 'Parallel operations count'
complete -c nsfw -f -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a 'max_cache_size_mb' -d 'Max cache size in MB'

# doctor command
complete -c nsfw -f -n '__fish_seen_subcommand_from doctor' -s h -l help -d 'Show help'

# completion command
complete -c nsfw -f -n '__fish_seen_subcommand_from completion' -a 'powershell' -d 'PowerShell completions'
complete -c nsfw -f -n '__fish_seen_subcommand_from completion' -a 'bash' -d 'Bash completions'
complete -c nsfw -f -n '__fish_seen_subcommand_from completion' -a 'zsh' -d 'Zsh completions'
complete -c nsfw -f -n '__fish_seen_subcommand_from completion' -a 'fish' -d 'Fish completions'
complete -c nsfw -f -n '__fish_seen_subcommand_from completion' -s h -l help -d 'Show help'

# upgrade command
complete -c nsfw -f -n '__fish_seen_subcommand_from upgrade' -s y -l yes -d 'Skip confirmation prompt'
complete -c nsfw -f -n '__fish_seen_subcommand_from upgrade' -s h -l help -d 'Show help'

# export command
complete -c nsfw -f -n '__fish_seen_subcommand_from export' -s o -l output -d 'Output file path' -r
complete -c nsfw -f -n '__fish_seen_subcommand_from export' -s f -l format -d 'Output format' -a 'json toml'
complete -c nsfw -f -n '__fish_seen_subcommand_from export' -s h -l help -d 'Show help'

# import command
complete -c nsfw -f -n '__fish_seen_subcommand_from import' -s y -l yes -d 'Skip confirmation prompt'
complete -c nsfw -f -n '__fish_seen_subcommand_from import' -s h -l help -d 'Show help'
# Suggest JSON and TOML files for import
complete -c nsfw -n '__fish_seen_subcommand_from import' -a '(__fish_complete_suffix .json)' -d 'JSON file'
complete -c nsfw -n '__fish_seen_subcommand_from import' -a '(__fish_complete_suffix .toml)' -d 'TOML file'
