// https://doc.rust-lang.org/stable/rust-by-example/error/multiple_error_types/boxing_errors.html

type Er = Box<dyn std::error::Error>;

#[derive(Debug)]
struct CustomError;
impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "custom error is here")
    }
}

impl std::error::Error for CustomError { }

fn mayfail(i: i32) -> Result<(), Er> {
    if i > 10 { return Err(CustomError.into()); }
    if i > 0 { return Ok(()); }
    return Err("oops".into());
}

fn main() {
    println!("-1: {:?}", mayfail(-1));
    println!("1: {:?}", mayfail(1));
}
