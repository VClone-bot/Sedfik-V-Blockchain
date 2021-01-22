use blocklib::*;

fn main() {
    let block = Block::new(0, "Premier bloc".to_owned(), 0, 0, vec![0; 32]);
    println!("{:?}", &block);
}
