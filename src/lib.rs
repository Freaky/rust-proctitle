#![deny(missing_docs)]

//! # Cross-platform process titles.
//!
//! `proctitle` attempts to expose the closest safe approximation of the BSD
//! `setproctitle()` function on the platforms it supports.
//!
//! This can be useful if you wish to expose some internal state to `top` or `ps`,
//! or to help an administrator distinguish between multiple instances of your
//! program.
//!
//! ```rust
//! # fn perform_task(_x: &str) { }
//! # fn main() {
//! use proctitle::set_title;
//! let tasks = ["frobrinate", "defroogle", "hodor", "bork"];
//!
//! for task in &tasks {
//!    set_title(format!("example: {}", task));
//!    perform_task(task);
//! }
//! set_title("example: idle");
//! # }
//! ```
//!
//! On Linux or a BSD you could then watch `top` or `ps` and see the process name
//! change as it works:
//!
//! ```sh
//! -% cmd &
//! [1] 8515
//! -% ps $!
//!  PID TT  STAT     TIME COMMAND
//! 8515  4  S+    0:00.06 example: defroggle (cmd)
//! ```
//!
//! ## Supported Platforms
//!
//! ### BSD
//!
//! On BSDs, `setproctitle()` is used, and should pretty much Just Work.  Use
//! `top -a` to see titles.
//!
//! ### Linux
//!
//! `proctitle` uses `prctl(PR_SET_NAME)` to name the current thread, with a
//! truncation limit of 15 bytes.
//!
//! More BSD-ish process-global changes are possible by modifying the process
//! environment, but this is not yet supported because it's wildly unsafe.
//!
//! ### Windows
//!
//! `SetConsoleTitleW()` is used to set a title for the console, if any.
//!
//! In case there is no console (for example, a system service), a dummy named
//! event handle is also created.  This can be found via tools such as Process
//! Explorer (View ⮕ Lower Pane View ⮕ Handles) and Process Hacker
//! (Properties ⮕ Handles).
//!
//! ### Everything Else
//!
//! Unsupported platforms merely receive a stub function that does nothing.

#[cfg(any(
    target_os = "freebsd",
    target_os = "hardenedbsd",
    target_os = "dragonflybsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod imp {
    use std::ffi::CString;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    /// Set a process title, or some approximation of it, if possible.
    pub fn set_title<T: AsRef<OsStr>>(title: T) {
        if let Ok(title) = CString::new(title.as_ref().to_owned().as_bytes()) {
            unsafe {
                setproctitle(b"-%s\0".as_ptr(), title.as_ptr());
            }
        }
    }

    #[link(name = "c")]
    extern "C" {
        fn setproctitle(fmt: *const u8, ...);
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use libc;
    use std::ffi::CString;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    /// Set a process title, or some approximation of it, if possible.
    pub fn set_title<T: AsRef<OsStr>>(title: T) {
        if let Ok(title) = CString::new(title.as_ref().to_owned().as_bytes()) {
            unsafe { libc::prctl(libc::PR_SET_NAME, title.as_ptr(), 0, 0, 0) };
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::Mutex;

    use lazy_static::lazy_static;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::synchapi::CreateEventW;
    use winapi::um::wincon::SetConsoleTitleW;
    use winapi::um::winnt::HANDLE;

    struct NamedHandle(HANDLE);
    unsafe impl Send for NamedHandle {}

    impl From<Vec<u16>> for NamedHandle {
        fn from(t: Vec<u16>) -> Self {
            assert!(t.ends_with(&[0]));

            Self(unsafe { CreateEventW(std::ptr::null_mut(), 1, 0, t.as_ptr()) })
        }
    }

    impl Drop for NamedHandle {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    lazy_static! {
        static ref EVENT_HANDLE: Mutex<Option<NamedHandle>> = Mutex::new(None);
    }

    /// Set a process title, or some approximation of it, if possible.
    pub fn set_title<T: AsRef<OsStr>>(title: T) {
        // Windows doesn't appear to have a userspace mechanism to name the current
        // process.
        //
        // Try to set a console title, and in case we're not attached to one,
        // follow PostgreSQL's lead and create a named event handle that can be
        // found in Process Explorer, Process Hacker, etc.
        let mut t: Vec<u16> = title.as_ref().encode_wide().take(1024).collect();
        t.push(0);

        unsafe { SetConsoleTitleW(t.as_ptr()) };

        EVENT_HANDLE
            .lock()
            .expect("event handle lock")
            .replace(NamedHandle::from(t));
    }
}

#[cfg(not(any(
    target_os = "freebsd",
    target_os = "hardenedbsd",
    target_os = "dragonflybsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "linux",
    target_os = "windows"
)))]
mod imp {
    use std::ffi::OsStr;

    /// Set a process title, or some approximation of it, if possible.
    pub fn set_title<T: AsRef<OsStr>>(_title: T) {}
}

pub use imp::*;
