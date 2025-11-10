use {relib_derive::Transfer, relib_module::__internal::TransferToHost, relib_shared::Transfer};

fn _test() {
  unsafe {
    let tuple = (1, "2", 3.0);
    let tuple2 = (1, "2", 3.0, true);
    let tuple3 = (1,);
    Transfer::<TransferToHost>::transfer(&(tuple, tuple2, tuple3), ());

    let array = [1, 2, 3];
    Transfer::<TransferToHost>::transfer(&array, ());
  }

  #[derive(Transfer)]
  struct TestDeriveStruct(i32);

  #[derive(Transfer)]
  enum TestDeriveEnum {
    Variant1,
    Variant2,
  }

  #[derive(Transfer)]
  struct TestDeriveStruct2 {
    field: i32,
    field2: Vec<i32>,
  }

  let _test = TestDeriveStruct(1);
  unsafe { Transfer::<TransferToHost>::transfer(&_test, ()) };

  let _test = TestDeriveEnum::Variant1;
  unsafe { Transfer::<TransferToHost>::transfer(&_test, ()) };

  let _test = TestDeriveStruct2 {
    field: 1,
    field2: vec![1, 2, 3],
  };
  unsafe { Transfer::<TransferToHost>::transfer(&_test, ()) };
}
