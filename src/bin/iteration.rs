fn iter() {
    let xs = [2, 3, 4, 5];
    // fails: iter::Iterator trait not implemented for arrays.
    // for x in xs { println!("x: {}", x); }

    // works, because slices are iterators.
    for x in &xs { println!("x: {}", x); }

    // also works
    for x in xs.iter() { println!("x: {}", x); }
}

fn range() {
    let xs = 1..5;
    println!("can print range xs: {:?}", xs);
    // implicit call to .into_iter()
    for x in xs { println!("range x: {}", x); }
    // can't print range anymore because moved due to implicit call.
    // println!("can't print after using it: {:?}", xs);

    let ys = 5..10;
    println!("can print range ys: {:?}", ys);
    // works if you clone
    for y in ys.clone() { println!("range y: {}", y); }
    println!("can print range ys: {:?}", ys);

    // question: is there any way to reuse a range without cloning? or are
    // ranges lightweight?
}

fn main() {
    iter();
    range();
}
