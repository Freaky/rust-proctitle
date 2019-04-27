# proctitle

[![Build Status](https://travis-ci.org/Freaky/rust-proctitle.svg?branch=master)](https://travis-ci.org/Freaky/rust-proctitle)

## Cross-platform process titles.

`proctitle` attempts to expose the closest safe approximation of the BSD
`setproctitle()` function on the platforms it supports.

This can be useful if you wish to expose some internal state to `top` or `ps`,
or to help an administrator distinguish between multiple instances of your
program.

```rust
use proctitle::set_title;
let tasks = ["frobrinate", "defroogle", "hodor", "bork"];

for task in &tasks {
   set_title(format!("example: {}", task));
   perform_task(task);
}
set_title("example: idle");
```

On Linux or a BSD you could then watch `top` or `ps` and see the process name
change as it works:

```sh
-% cmd &
[1] 8515
-% ps $!
 PID TT  STAT     TIME COMMAND
8515  4  S+    0:00.06 example: defroggle (cmd)
```

### Supported Platforms

#### BSD

On BSDs, `setproctitle()` is used, and should pretty much Just Work.  Use
`top -a` to see titles.

#### Linux

`proctitle` uses `prctl(PR_SET_NAME)` to name the current thread, with a
truncation limit of 15 bytes.

More BSD-ish process-global changes are possible by modifying the process
environment, but this is not yet supported because it's wildly unsafe.

#### Windows

`SetConsoleTitleW()` is used to set a title for the console, if any.

In case there is no console (for example, a system service), a dummy named
event handle is also created.  This can be found via tools such as Process
Explorer (View ⮕ Lower Pane View ⮕ Handles) and Process Hacker
(Properties ⮕ Handles).

#### Everything Else

Unsupported platforms merely receive a stub function that does nothing.

