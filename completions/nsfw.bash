#!/usr/bin/env bash
# Bash completion script for NSFW (Nix Subsystem for Windows)
# Install: source this file or copy to /etc/bash_completion.d/nsfw

_nsfw_completions() {
    local cur prev words cword
    _init_completion || return

    # Main commands
    local commands="search install remove list info update setup cache doctor completion config upgrade export import"

    # Cache subcommands
    local cache_commands="stats clear rebuild"

    # Config subcommands
    local config_commands="show get set reset path keys"

    # Config keys
    local config_keys="cache_ttl_days default_wrapper_type auto_update_channels install_location verbose_output disable_colors parallel_jobs max_cache_size_mb"

    # Global options
    local global_opts="--help --version --verbose"

    # Get the command (first non-option argument)
    local command=""
    local i
    for ((i=1; i < cword; i++)); do
        if [[ ${words[i]} != -* ]]; then
            command=${words[i]}
            break
        fi
    done

    # Get the subcommand for cache/config
    local subcommand=""
    if [[ "$command" == "cache" || "$command" == "config" ]]; then
        for ((i=2; i < cword; i++)); do
            if [[ ${words[i]} != -* ]]; then
                subcommand=${words[i]}
                break
            fi
        done
    fi

    # Completion logic
    case "$prev" in
        nsfw)
            COMPREPLY=($(compgen -W "$commands $global_opts" -- "$cur"))
            return
            ;;
        search)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--limit --format --help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        install|add)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--yes --help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        remove|uninstall)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--yes --help" -- "$cur"))
                    ;;
                *)
                    # Could suggest installed packages here
                    ;;
            esac
            return
            ;;
        list|ls)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--detailed --format --help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        info)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        update)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        setup)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--yes --interactive --help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        cache)
            if [[ -z "$subcommand" ]]; then
                COMPREPLY=($(compgen -W "$cache_commands --help" -- "$cur"))
            else
                case "$cur" in
                    -*)
                        COMPREPLY=($(compgen -W "--help" -- "$cur"))
                        ;;
                esac
            fi
            return
            ;;
        config)
            if [[ -z "$subcommand" ]]; then
                COMPREPLY=($(compgen -W "$config_commands --help" -- "$cur"))
            elif [[ "$subcommand" == "get" || "$subcommand" == "set" ]]; then
                # Suggest config keys
                COMPREPLY=($(compgen -W "$config_keys" -- "$cur"))
            else
                case "$cur" in
                    -*)
                        COMPREPLY=($(compgen -W "--help" -- "$cur"))
                        ;;
                esac
            fi
            return
            ;;
        doctor)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        completion)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--help" -- "$cur"))
                    ;;
                *)
                    COMPREPLY=($(compgen -W "powershell bash zsh fish" -- "$cur"))
                    ;;
            esac
            return
            ;;
        upgrade)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--yes --help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        export)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--output --format --help" -- "$cur"))
                    ;;
            esac
            return
            ;;
        import)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "--yes --help" -- "$cur"))
                    ;;
                *)
                    # Suggest files
                    COMPREPLY=($(compgen -f -X '!*.@(json|toml)' -- "$cur"))
                    ;;
            esac
            return
            ;;
        --format)
            COMPREPLY=($(compgen -W "text json toml" -- "$cur"))
            return
            ;;
        --output|-o)
            # Suggest file completion
            COMPREPLY=($(compgen -f -- "$cur"))
            return
            ;;
    esac

    # Default: suggest commands if no command yet
    if [[ -z "$command" ]]; then
        COMPREPLY=($(compgen -W "$commands $global_opts" -- "$cur"))
    fi
}

# Register completion function
complete -F _nsfw_completions nsfw
