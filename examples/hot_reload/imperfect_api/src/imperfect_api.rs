relib_interface::include_imports!();

#[repr(C)]
#[derive(Debug)]
pub struct Entity(u64);

impl Drop for Entity {
  fn drop(&mut self) {
    unsafe { gen_imports::despawn_entity_from_not_perfect_parallel_universe(self.0) };
    println!("entity {} despawned (imperfect_api)", self.0);
  }
}

pub fn spawn() -> Entity {
  let id = unsafe { gen_imports::spawn_entity_from_not_perfect_parallel_universe() };
  println!("entity {} spawned (imperfect_api)", id);
  Entity(id)
}
