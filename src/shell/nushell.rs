pub const SNIPPET: &str = r#"# metrocity - terminal screensaver (nushell integration)
# https://github.com/Itz-Agasta/metrocity
#
# Requires Python 3. Nushell's job model places background processes in a
# separate process group with SIGTTIN blocked, which prevents TUI apps from
# reading keyboard input. A small Python helper joins the shell's foreground
# process group (mimicking bash's `set +m`) to restore input.

# Install the Python helper to the nushell config directory.
$env.METROCITY_HELPER = ($nu.default-config-dir | path join "metrocity-helper.py")
try {
    '#!/usr/bin/env python3
import os, sys, time
try:
    timeout = int(sys.argv[1])
    args = ["metrocity"]
    if len(sys.argv) > 2:
        args += ["--scene", sys.argv[2]]
    tty = os.open("/dev/tty", os.O_RDWR)
    try: os.setpgid(0, os.tcgetpgrp(tty))
    except OSError: pass
    os.dup2(tty, 1)
    os.dup2(tty, 2)
    time.sleep(timeout)
    os.execvp("metrocity", args)
except Exception:
    sys.exit(1)' | save --force $env.METROCITY_HELPER
}

# Sentinel PID; -1 means no timer armed.
$env.METROCITY_TIMER_PID = -1

def --env _metrocity_cancel [] {
    if $env.METROCITY_TIMER_PID >= 0 {
        try { ^kill $env.METROCITY_TIMER_PID err> /dev/null }
        $env.METROCITY_TIMER_PID = -1
    }
}

def --env _metrocity_schedule [] {
    _metrocity_cancel
    if (which metrocity | is-empty) { return }
    let helper = ($env.METROCITY_HELPER? | default "")
    if ($helper | is-empty) { return }
    if not ($helper | path exists) { return }
    let timeout = ($env.METROCITY_TIMEOUT? | default 120)
    let scene = ($env.METROCITY_SCENE? | default "")
    $env.METROCITY_TIMER_PID = try {
        if ($scene | is-not-empty) {
            ^sh -c 'python3 "$1" "$2" "$3" & echo $!' _ $helper $timeout $scene | str trim | into int
        } else {
            ^sh -c 'python3 "$1" "$2" & echo $!' _ $helper $timeout | str trim | into int
        }
    } catch {
        -1
    }
}

# Append our hooks to any the user already has, so we never clobber their
# pre_prompt / pre_execution setup.
$env.config.hooks = ($env.config.hooks | upsert pre_prompt { |o|
    ($o.pre_prompt? | default []) ++ [{|| _metrocity_schedule }]
} | upsert pre_execution { |o|
    ($o.pre_execution? | default []) ++ [{|| _metrocity_cancel }]
})
"#;
