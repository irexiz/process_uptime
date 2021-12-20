# Description

Library fetching process uptime via
`ps -o etimes -p <PID> --no-headers`

If ps is restricted (as is the case on some devices), library fallbacks to fetching process
uptime from `/proc/{pid}/stat` starttime column
