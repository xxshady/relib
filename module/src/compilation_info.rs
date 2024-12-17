use relib_internal_shared::Str;

#[unsafe(export_name = "__RELIB__CRATE_COMPILATION_INFO__")]
static INFO: Str = Str::const_from(crate_compilation_info::get!());
