use std::path::PathBuf;

use some_other_file::some_function;

mod some_other_file;

fn main() {
    some_function("a");

    // should be an error label below
    some_function

    some_function("b");
}