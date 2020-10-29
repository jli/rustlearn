fn static_array_bounds_check() {
    let xs = [1, 2, 3];
    let i = 3;
    // gives error, huh, unexpected.
    // https://doc.rust-lang.org/book/ch03-02-data-types.html says it wouldn't.
    // println!("xs[3]: {}", xs[i]);
    println!("xs[2]: {}", xs[i-1]);
}

fn main() {
    static_array_bounds_check();
}
