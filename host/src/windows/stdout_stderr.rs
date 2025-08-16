use std::{ffi::c_void, io, mem::size_of, sync::Mutex, thread};

use crate::windows::imports::{
  CreatePipe, GetStdHandle, SetStdHandle, CloseHandle, ReadFile, WriteFile, STD_OUTPUT_HANDLE,
  STD_ERROR_HANDLE, TRUE, FALSE, SECURITY_ATTRIBUTES, HANDLE,
};

static STDOUT_SYNC: Mutex<Option<Sync>> = Mutex::new(None);
static STDERR_SYNC: Mutex<Option<Sync>> = Mutex::new(None);

#[derive(Clone, Copy)]
struct SendHandle(HANDLE);
impl SendHandle {
  pub fn handle(self) -> HANDLE {
    self.0
  }
}
unsafe impl Send for SendHandle {}

struct Sync {
  _reader_thread: thread::JoinHandle<()>,
}

pub fn try_init() {
  let mut stdout_sync = STDOUT_SYNC.lock().unwrap();
  if stdout_sync.is_none() {
    *stdout_sync = Some(Sync::new(STD_OUTPUT_HANDLE));
  }

  let mut stderr_sync = STDERR_SYNC.lock().unwrap();
  if stderr_sync.is_none() {
    *stderr_sync = Some(Sync::new(STD_ERROR_HANDLE));
  }
}

impl Sync {
  fn new(handle_type: u32) -> Self {
    let mut read_pipe = std::ptr::null_mut();
    let mut write_pipe = std::ptr::null_mut();

    let mut security_attributes = SECURITY_ATTRIBUTES {
      nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
      lpSecurityDescriptor: std::ptr::null_mut(),
      bInheritHandle: TRUE,
    };

    let result =
      unsafe { CreatePipe(&mut read_pipe, &mut write_pipe, &mut security_attributes, 0) };
    if result == FALSE {
      panic!("Failed to create pipe: {}", io::Error::last_os_error());
    }

    let original_handle = unsafe { GetStdHandle(handle_type) };
    if original_handle == std::ptr::null_mut() || original_handle == usize::MAX as *mut c_void {
      panic!(
        "Failed to get original stdout/stderr handle: {}",
        io::Error::last_os_error()
      );
    }

    let result = unsafe { SetStdHandle(handle_type, write_pipe) };
    if result == FALSE {
      panic!(
        "Failed to set stdout/stderr handle: {}",
        io::Error::last_os_error()
      );
    }

    let read_pipe = SendHandle(read_pipe);
    let write_pipe = SendHandle(write_pipe);
    let original_handle = SendHandle(original_handle);

    let reader_thread = thread::spawn(move || {
      let mut buffer = [0; 1024];
      loop {
        let mut bytes_read = 0;
        let result = unsafe {
          ReadFile(
            read_pipe.handle(),
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len() as u32,
            &mut bytes_read,
            std::ptr::null_mut(),
          )
        };

        if result == FALSE || bytes_read == 0 {
          break;
        }

        let mut bytes_written = 0;
        let result = unsafe {
          WriteFile(
            original_handle.handle(),
            buffer.as_ptr() as *const c_void,
            bytes_read,
            &mut bytes_written,
            std::ptr::null_mut(),
          )
        };

        if result == FALSE {
          break;
        }
      }
      unsafe {
        CloseHandle(read_pipe.handle());
        CloseHandle(write_pipe.handle());
        SetStdHandle(handle_type, original_handle.handle());
      }
    });

    Self {
      _reader_thread: reader_thread,
    }
  }
}
