pub fn why() {
  // this crate is an example local dependency of shared crate
  // when it's modified shared crate should be rebuilt
  //
  // you can run host crate and then modify this crate to see that
  //
  // this is important because by default cargo won't re-run build.rs
  // when non-build dependency is modified

  // it also should work recursively:
  shared_dependency_of_dependency::why();
}
