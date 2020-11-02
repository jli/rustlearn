// The str type, also called a 'string slice', is the most primitive string
// type. It is usually seen in its borrowed form, &str. It is also the type of
// string literals, &'static str.

const GLOB_STR: &str = "global str";

fn strs() {
    // uh, how does && work, and how come they all print? maybe just println.
    let str1: &str = "hi";
    let str1ref: &&str = &str1;
    let str1refref: &&&str = &&str1;
    let s1 = str1.to_string();
    let s1ref = &s1;
    let s1refref = &&s1;

    println!("str1 (&str): {:?} ptr: {:?} len: {:?}", str1, str1.as_ptr(), str1.len());
    println!("str1 ref (&&str): {}", str1ref);
    println!("str1 ref ref (&&&str): {}", str1refref);
    println!("s1 (String): {}", s1);
    println!("s1 ref: {}", s1ref);
    println!("s1 ref ref: {}", s1refref);
    println!("s1 ref ref **: {}", **s1refref);
    println!("global string: {}", GLOB_STR);
    println!("global string ref: {}", &GLOB_STR);
}

fn str_slice_bad() {
    let s = String::from("123");
    // causes panic because 10 is out of bounds
    let str = &s[0..10];
    println!("{} {}", s, str);
}


fn main() {
    strs();
    str_slice_bad();
}
