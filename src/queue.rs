use std::ptr;

mod win32 {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use libc;

    pub type HANDLE = *mut libc::c_void;
    pub type BOOL = i32;

    #[cfg(target_pointer_width = "32")]
    pub type ULONG_PTR = u32;
    #[cfg(target_pointer_width = "64")]
    pub type ULONG_PTR = u64;

    pub const INVALID_HANDLE_VALUE: HANDLE = 0xFFFFFFFFFFFFFFFF as HANDLE;
    pub const INFINITE: u32 = 0xFFFFFFFF;
    pub const NULL_HANDLE: HANDLE = 0 as HANDLE;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct OVERLAPPED {
        pub Internal: ULONG_PTR,
        pub InternalHigh: ULONG_PTR,
        pub Offset: u32,
        pub OffsetHigh: u32,
        pub hEvent: HANDLE,
    }

    #[link(name = "kernel32")]
    extern "stdcall" {
        pub fn CreateIoCompletionPort (
            FileHandle: HANDLE,             // IN
            ExistingCompletionPort: HANDLE, // IN OPT
            CompletionKey: ULONG_PTR,       // IN
            NumberOfConcurrentThreads: u32  // IN
        ) -> HANDLE;

        pub fn GetQueuedCompletionStatus (
            CompletionPort: HANDLE,             // IN
            lpNumberOfBytes: *mut u32,          // OUT
            lpCompletionKey: *mut ULONG_PTR,    // OUT
            lpOverlapped: *mut *mut OVERLAPPED, // OUT
            dwMilliseconds: u32                 // IN
        ) -> BOOL;

        pub fn PostQueuedCompletionStatus (
            CompletionPort: HANDLE,             // IN
            dwNumberOfBytesTransferred: u32,    // IN
            dwCompletionKey: ULONG_PTR,         // IN
            lpOverlapped: *mut OVERLAPPED       // IN OPT
        ) -> BOOL;

        pub fn CloseHandle (
            hObject: HANDLE // IN
        ) -> BOOL;
    }
}

#[derive(Copy, Clone)]
pub struct Status {
    byte_count: u32,
    completion_key: u64,
    overlapped: Option<win32::OVERLAPPED>
}

pub struct Port {
    handle: win32::HANDLE
}

impl Port {
    pub fn new () -> Option<Port> {
        return Port::new_capped(0);
    }

    pub fn new_capped (max_threads: u32) -> Option<Port> {
        let handle;
        unsafe {
            handle = win32::CreateIoCompletionPort(
                win32::INVALID_HANDLE_VALUE,
                win32::NULL_HANDLE,
                0,
                max_threads
            );
        }

        if handle.is_null() {
            return None;
        }

        return Some(Port { handle: handle });
    }

    pub fn get_status (&mut self) -> Option<Status> {
        return self.get_status_timeout(win32::INFINITE);
    }

    pub fn get_status_timeout (&mut self, timeout_ms: u32) -> Option<Status> {
        let mut status: Status = Status {
            byte_count: 0,
            completion_key: 0,
            overlapped: None
        };
        let mut overlapped: *mut win32::OVERLAPPED = ptr::null_mut();

        let success;
        unsafe {
            success = win32::GetQueuedCompletionStatus(
                self.handle,
                &mut status.byte_count as *mut u32,
                &mut status.completion_key as *mut u64,
                &mut overlapped as *mut *mut win32::OVERLAPPED,
                timeout_ms
            );
        }

        if success == 0 {
            return None;
        }

        if !overlapped.is_null() {
            unsafe {
                status.overlapped = Some(*overlapped);
            }
        }

        return Some(status);
    }

    pub fn post_status (&mut self, mut status: Status) -> bool {
        // Get overlapped raw pointer
        let mut overlapped: *mut win32::OVERLAPPED = ptr::null_mut();
        if let Some(ref mut o) = status.overlapped {
            overlapped = o as *mut win32::OVERLAPPED;
        }

        // Call system API
        let success;
        unsafe{
            success = win32::PostQueuedCompletionStatus(
                self.handle,
                status.byte_count,
                status.completion_key,
                overlapped
            );
        }

        return success == 0;
    }

    pub fn close (self) {
        unsafe {
            win32::CloseHandle(self.handle);
        }
    }
}
