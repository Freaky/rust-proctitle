#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "bitrig",
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

    #[test]
    fn set_title_sets_name() {
        use libc;
        set_title("abcdefghijklmnopqrstu");

        let mut buf = [0u8; 16];
        unsafe { libc::prctl(libc::PR_GET_NAME, buf.as_mut_ptr(), 0, 0, 0) };
        assert_eq!(&buf, b"abcdefghijklmno\0");
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
            if !self.0.is_null() {
                unsafe { CloseHandle(self.0) };
            }
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

    #[test]
    fn set_title_sets_console_title_and_makes_a_handle() {
        let title = "Pinkle, squirmy, blib, blab, blob";
        set_title(title);

        let mut t: Vec<u16> = std::ffi::OsString::from(title).encode_wide().collect();
        t.push(0);
        let mut buf = vec![0; t.len()];
        let len = unsafe { winapi::um::wincon::GetConsoleTitleW(buf.as_mut_ptr(), buf.len() as u32) };

        assert_eq!(len, title.len() as u32, "length mismatch");
        assert_eq!(buf, t, "buffer mismatch");
        assert!(EVENT_HANDLE.lock().unwrap().is_some(), "event handle missing");
    }
}

#[cfg(not(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "bitrig",
    target_os = "linux",
    target_os = "windows"
)))]
mod imp {
    use std::ffi::OsStr;

    /// Set a process title, or some approximation of it, if possible.
    pub fn set_title<T: AsRef<OsStr>>(_title: T) {}
}

pub use imp::*;

// This races against the SetConsoleTitle() tests on Windows
#[cfg(not(windows))]
#[test]
fn set_title_is_at_least_callable() {
    set_title("What was it like being a hamster?");
    set_title(String::from("It was better than being a chicken."));
    set_title(std::ffi::OsString::from("Have you seen the size of an egg?"));
}
