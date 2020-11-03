use regex::Regex;
use lazy_static::lazy_static;

fn get_date(s: &str) -> &str {
    let re = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();
    re.captures(s).unwrap().get(1).unwrap().as_str()
}

fn match_brackets(s: &str) -> &str {
    let re = Regex::new(r"\[(\d+)\]").unwrap();
    re.captures(s).unwrap().get(1).unwrap().as_str()
}

// lazy_static pattern to avoid recompiling regexes
fn hex_static(s: &str) -> &str {
    lazy_static! {
        static ref HEX: Regex = Regex::new(r"([[:xdigit:]]+)").unwrap();
    }
    HEX.captures(s).unwrap().get(1).unwrap().as_str()
}
fn main() {
    println!("gd(2020-10-28) = {}", get_date("2020-10-28"));
    println!("mb([123]) = {}", match_brackets("[123]"));
    println!("hex(hi DEADBEEFF00F yay) = {}", hex_static("hi DEADBEEFF00F yay"));
}
