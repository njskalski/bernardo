use std::path::PathBuf;

use some_other_file::some_function;

mod some_other_file;

fn main() {
    some_function("a");

    //

    some_function("b");
}