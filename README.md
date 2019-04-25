# proctitle

A (hopefully) safe interface to setting the process name, or something like it.

On FreeBSD, HardenedBSD, DragonFlyBSD, and NetBSD this calls `setproctitle()`.
In `top`, press `a` to view titles.

On Linux it calls `prctl(PR_SET_NAME)`, which truncates to 15 bytes and sets the
name of the current thread: if you want it to name the process, call it before
spawning new threads.

On Windows it creates a named event handle, which can be found in the handle
list using a tool such as Process Explorer or Process Hacker.

On unsupported platforms it is a stub implementation which does nothing.

## Usage

```rust
use proctitle;

proctitle::set_title("Bleep bloop");
```

`set_title()` accepts `AsRef<OsStr>`, which should cover most types you'd like
to use.  
