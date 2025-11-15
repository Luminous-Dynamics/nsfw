#compdef nsfw
# Zsh completion script for NSFW (Nix Subsystem for Windows)
# Install: Copy to a directory in $fpath (e.g., /usr/local/share/zsh/site-functions/_nsfw)

_nsfw() {
    local curcontext="$curcontext" state line
    typeset -A opt_args

    # Main command completion
    _arguments -C \
        '(-h --help)'{-h,--help}'[Show help information]' \
        '(-v --verbose)'{-v,--verbose}'[Enable verbose logging]' \
        '--version[Show version information]' \
        '1: :_nsfw_commands' \
        '*::arg:->args'

    case $state in
        args)
            case $words[1] in
                search|find)
                    _arguments \
                        '(-l --limit)'{-l,--limit}'[Maximum number of results]:limit:' \
                        '(-f --format)'{-f,--format}'[Output format]:format:(text json)' \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:query:'
                    ;;
                install|add)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                        '--dry-run[Show what would be installed without actually installing]' \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:package:'
                    ;;
                remove|uninstall)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                        '--dry-run[Show what would be removed without actually removing]' \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:package:_nsfw_installed_packages'
                    ;;
                list|ls)
                    _arguments \
                        '(-d --detailed)'{-d,--detailed}'[Show detailed information]' \
                        '(-f --format)'{-f,--format}'[Output format]:format:(text json)' \
                        '(-h --help)'{-h,--help}'[Show help]'
                    ;;
                info)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:package:'
                    ;;
                update)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]'
                    ;;
                setup)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompts]' \
                        '(-i --interactive)'{-i,--interactive}'[Interactive mode]' \
                        '(-h --help)'{-h,--help}'[Show help]'
                    ;;
                cache)
                    _arguments '1: :_nsfw_cache_commands'
                    ;;
                config)
                    _arguments '1: :_nsfw_config_commands' \
                               '2: :_nsfw_config_keys'
                    ;;
                doctor)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]'
                    ;;
                completion)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:shell:(powershell bash zsh fish)'
                    ;;
                upgrade)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                        '--dry-run[Show what would be upgraded without actually upgrading]' \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '::package:'
                    ;;
                export)
                    _arguments \
                        '(-o --output)'{-o,--output}'[Output file path]:file:_files' \
                        '(-f --format)'{-f,--format}'[Output format]:format:(json toml)' \
                        '(-h --help)'{-h,--help}'[Show help]'
                    ;;
                import)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                        '--dry-run[Show what would be imported without actually installing]' \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:file:_files -g "*.{json,toml}"'
                    ;;
            esac
            ;;
    esac
}

_nsfw_commands() {
    local -a commands
    commands=(
        'search:Search for packages in nixpkgs'
        'find:Alias for search'
        'install:Install a package'
        'add:Alias for install'
        'remove:Remove an installed package'
        'uninstall:Alias for remove'
        'list:List installed packages'
        'ls:Alias for list'
        'info:Show information about a package'
        'update:Update the package database'
        'setup:Setup WSL2 and Nix environment'
        'cache:Manage package cache'
        'doctor:Diagnose system health and configuration'
        'completion:Install shell completions'
        'config:Manage configuration settings'
        'upgrade:Upgrade installed packages to latest version'
        'export:Export installed packages to a file'
        'import:Import and install packages from a file'
    )
    _describe -t commands 'nsfw command' commands
}

_nsfw_cache_commands() {
    local -a cache_commands
    cache_commands=(
        'stats:Show cache statistics'
        'clear:Clear the package cache'
        'rebuild:Rebuild the package cache'
    )
    _describe -t cache-commands 'cache command' cache_commands
}

_nsfw_config_commands() {
    local -a config_commands
    config_commands=(
        'show:Show all configuration settings'
        'get:Get a specific configuration value'
        'set:Set a configuration value'
        'reset:Reset configuration to defaults'
        'path:Show configuration file path'
        'keys:List all available configuration keys'
    )
    _describe -t config-commands 'config command' config_commands
}

_nsfw_config_keys() {
    local -a config_keys
    config_keys=(
        'cache_ttl_days:Number of days before package cache expires'
        'default_wrapper_type:Default wrapper type (console/gui/vbs)'
        'auto_update_channels:Automatically update Nix channels'
        'install_location:Custom installation location for wrappers'
        'verbose_output:Enable verbose output for all commands'
        'disable_colors:Disable colored terminal output'
        'parallel_jobs:Number of parallel operations for batch commands'
        'max_cache_size_mb:Maximum cache size in MB'
    )
    _describe -t config-keys 'config key' config_keys
}

_nsfw_installed_packages() {
    # Could query nsfw list here for installed packages
    # For now, just allow any input
    _message 'package name'
}

_nsfw "$@"
