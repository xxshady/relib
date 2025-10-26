use {
  main_contract::SharedImports,
  std::{
    cell::{Cell, RefCell},
    collections::HashSet,
  },
};

relib_interface::include_imports!(gen_imports, "main_module");
pub use gen_imports::init_imports as init_shared_imports;
use gen_imports::ModuleImportsImpl;

thread_local! {
  static ENTITIES: RefCell<HashSet<u64>> = Default::default();
  static ENTITY_ID_COUNTER: Cell<u64> = Cell::new(1);
}

pub fn despawn_leaked_entities() {
  ENTITIES.with_borrow_mut(|entities| {
    println!(
      "despawning leaked entities: {} (imperfect_api)",
      entities.len()
    );

    std::mem::take(entities);
  });
}

impl SharedImports for ModuleImportsImpl {
  fn spawn_entity_from_not_perfect_parallel_universe() -> u64 {
    let id = ENTITY_ID_COUNTER.get();
    ENTITY_ID_COUNTER.set(id + 1);

    ENTITIES.with_borrow_mut(|entities| {
      entities.insert(id);
    });

    id
  }

  fn despawn_entity_from_not_perfect_parallel_universe(id: u64) {
    ENTITIES.with_borrow_mut(|entities| {
      entities.remove(&id);
    });
  }
}
