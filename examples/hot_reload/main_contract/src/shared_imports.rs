// these imports are shared between main module and update module
pub trait SharedImports {
  fn spawn_entity_from_not_perfect_parallel_universe() -> u64;
  fn despawn_entity_from_not_perfect_parallel_universe(entity: u64);
}
