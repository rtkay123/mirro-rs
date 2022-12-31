
use builtin;
use str;

set edit:completion:arg-completer[mirro-rs] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'mirro-rs'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'mirro-rs'= {
            cand -o 'File to write mirrors to'
            cand --outfile 'File to write mirrors to'
            cand -e 'Number of mirrors to export [default: 50]'
            cand --export 'Number of mirrors to export [default: 50]'
            cand -v 'An order to view all countries'
            cand --view 'An order to view all countries'
            cand -s 'Default sort for exported mirrors'
            cand --sort 'Default sort for exported mirrors'
            cand -t 'Number of hours to cache mirrorlist for'
            cand --ttl 'Number of hours to cache mirrorlist for'
            cand -u 'URL to check for mirrors'
            cand --url 'URL to check for mirrors'
            cand --config 'Specify alternate configuration file'
            cand --timeout 'Connection timeout in seconds'
            cand -a 'How old (in hours) should the mirrors be since last synchronisation'
            cand --age 'How old (in hours) should the mirrors be since last synchronisation'
            cand -c 'Countries to search for mirrorlists'
            cand -p 'Filters to use on mirrorlists'
            cand --protocols 'Filters to use on mirrorlists'
            cand --completion-percent 'Set the minimum completion percent for the returned mirrors'
            cand -r 'Sort mirrorlists by download speed when exporting'
            cand --rate 'Sort mirrorlists by download speed when exporting'
            cand --ipv4 'Only return mirrors that support IPv4'
            cand --ipv6 'Only return mirrors that support IPv6'
            cand --isos 'Only return mirrors that host ISOs'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
    ]
    $completions[$command]
}
