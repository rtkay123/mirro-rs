_mirro-rs() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="mirro__rs"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        mirro__rs)
            opts="-o -e -v -s -t -u -r -i -d -a -c -p -h -V --outfile --export --view --sort --ttl --url --rate --timeout --include --direct --age --protocols --ipv4 --ipv6 --isos --completion-percent --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --outfile)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --export)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -e)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --view)
                    COMPREPLY=($(compgen -W "alphabetical mirror-count" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "alphabetical mirror-count" -- "${cur}"))
                    return 0
                    ;;
                --sort)
                    COMPREPLY=($(compgen -W "percentage delay duration score" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "percentage delay duration score" -- "${cur}"))
                    return 0
                    ;;
                --ttl)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --include)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --age)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --protocols)
                    COMPREPLY=($(compgen -W "https http rsync" -- "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -W "https http rsync" -- "${cur}"))
                    return 0
                    ;;
                --completion-percent)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _mirro-rs -o bashdefault -o default mirro-rs
