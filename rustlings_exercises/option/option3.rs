// option3.rs
// Make me compile! Execute `rustlings hint option3` for hints

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let x: Point = Point { x: 100, y: 200 };

    match x {
        ref p => println!("Co-ordinates are {},{} ", p.x, p.y),
    }
    // x; // Fix without deleting this line.

    match &x {
        p => println!("Co-ordinates are {},{} ", p.x, p.y),
    }
    x; // Fix without deleting this line.

    let y: Option<Point> = Some(Point { x: 100, y: 200 });

    match y {
        Some(ref p) => println!("Co-ordinates are {},{} ", p.x, p.y),
        _ => println!("no match"),
    }
    match &y {
        &Some(ref p) => println!("Co-ordinates are {},{} ", p.x, p.y),
        _ => println!("no match"),
    }
    y; // Fix without deleting this line.
}
