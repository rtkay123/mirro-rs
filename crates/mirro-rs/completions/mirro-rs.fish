complete -c mirro-rs -s o -l outfile -d 'File to write mirrors to' -r -F
complete -c mirro-rs -s e -l export -d 'Number of mirrors to export [default: 50]' -r
complete -c mirro-rs -s v -l view -d 'An order to view all countries' -r -f -a "{alphabetical	,mirror-count	}"
complete -c mirro-rs -s s -l sort -d 'Default sort for exported mirrors' -r -f -a "{percentage	,delay	,duration	,score	}"
complete -c mirro-rs -s t -l ttl -d 'Number of hours to cache mirrorlist for' -r
complete -c mirro-rs -s u -l url -d 'URL to check for mirrors' -r
complete -c mirro-rs -l timeout -d 'Connection timeout in seconds' -r
complete -c mirro-rs -s i -l include -d 'Extra CDNs to check for mirrors' -r
complete -c mirro-rs -s a -l age -d 'How old (in hours) should the mirrors be since last synchronisation' -r
complete -c mirro-rs -s c -d 'Countries to search for mirrorlists' -r
complete -c mirro-rs -s p -l protocols -d 'Filters to use on mirrorlists' -r -f -a "{https	,http	,rsync	}"
complete -c mirro-rs -l completion-percent -d 'Set the minimum completion percent for the returned mirrors' -r
complete -c mirro-rs -s r -l rate -d 'Sort mirrorlists by download speed when exporting'
complete -c mirro-rs -l ipv4 -d 'Only return mirrors that support IPv4'
complete -c mirro-rs -l ipv6 -d 'Only return mirrors that support IPv6'
complete -c mirro-rs -l isos -d 'Only return mirrors that host ISOs'
complete -c mirro-rs -s h -l help -d 'Print help information'
complete -c mirro-rs -s V -l version -d 'Print version information'
