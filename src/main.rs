use std::hash::{DefaultHasher, Hash, Hasher};

fn main() {
    let hola = 25;
    let mut hasher = DefaultHasher::new();
    hola.hash(&mut hasher);
    let value = hasher.finish();
    println!("Hello, world!, {:?}", value);
}
