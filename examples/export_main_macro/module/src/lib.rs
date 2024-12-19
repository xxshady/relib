#[main_export::main_export]
fn main() {
  println!("hello world");
}

// will fail to compile
// #[main_export::main_export]
// fn main() -> i32 {
//   println!("hello world");
// }
