fn loops_return_values() {
    let mut counter = 0;
    let x = loop {
        counter += 1;
        println!("counter: {}", counter);
        if counter >= 3 {
            break 10;
        }
    };
    println!("loop broke with: {}", x);
}

fn main() {
    loops_return_values();
}
