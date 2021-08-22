fn main() {
    let arr_small = [1,2];
    println!("arr small: {:?}", arr_small);

    // ok, these both. i guess because the String/Vec values are fixed-sized
    // pointers to the underlynig growable heap data structures?
    let mut arr_strings = [String::from("hi")];
    println!("arr strings1: {:?}", arr_strings);
    arr_strings[0].push_str(" there");
    println!("arr strings2: {:?}", arr_strings);

    let mut arr_vecs = [vec![1,2], vec![3,4]];
    println!("arr vecs1: {:?}", arr_vecs);
    arr_vecs[1].push(5);
    println!("arr vecs2: {:?}", arr_vecs);
}
