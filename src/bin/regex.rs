use regex::Regex;

fn get_date(s: &str) -> &str {
    let re = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();
    re.captures(s).unwrap().get(1).unwrap().as_str()
}

fn main() {
    println!("gd(2020-10-28) = {}", get_date("2020-10-28"));
}
