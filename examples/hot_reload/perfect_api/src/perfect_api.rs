#[repr(C)]
#[derive(Debug)]
pub struct Entity(u64);

impl Drop for Entity {
  fn drop(&mut self) {
    println!("entity {} despawned (perfect_api)", self.0);
  }
}

#[repr(C)]
#[derive(Debug)]
pub struct ApiState {
  id_counter: u64,
}

impl Default for ApiState {
  fn default() -> Self {
    Self { id_counter: 1 }
  }
}

pub fn spawn(state: &mut ApiState) -> Entity {
  let id = state.id_counter;
  state.id_counter += 1;

  println!("entity {} spawned (perfect_api)", id);

  Entity(id)
}
