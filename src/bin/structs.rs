#[derive(Debug)]
struct U {
    i: i32,
    s: String,
}

fn mod_u(u: &U) -> U {
    // update syntax requires value not reference.
    // copies values. fails if trying to reuse String value.
    U {
        s: String::from("new"),
        ..*u
    }
}

fn main() {
    let u = U { i: 100, s: String::from("string"), };
    println!("{:?}, {:?}", u, mod_u(&u));
}
