// ok, seems like stuff in src/bin needs to use the package prefix, and the
// stuff that's exported in the package is defined in lib.rs
use modlearn::utils;

fn main() {
    println!("Hello, world! 2+2={}", utils::add2(2));
}
