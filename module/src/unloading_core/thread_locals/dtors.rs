use std::cell::RefCell;

struct Destructors(RefCell<Vec<(*mut u8, extern "C" fn(*mut u8))>>);

// SAFETY: register & run will be called from the same thread
unsafe impl Send for Destructors {}
unsafe impl Sync for Destructors {}

static DESTRUCTORS: Destructors = Destructors(RefCell::new(Vec::new()));

pub unsafe fn register(obj: *mut u8, dtor: extern "C" fn(*mut u8)) {
  let mut dtors = DESTRUCTORS.0.borrow_mut();
  dtors.push((obj, dtor));
}

pub unsafe fn run() {
  loop {
    let mut dtors = DESTRUCTORS.0.borrow_mut();
    match dtors.pop() {
      Some((obj, dtor)) => {
        drop(dtors);
        dtor(obj);
      }
      None => {
        *dtors = Vec::new();
        break;
      }
    }
  }
}
