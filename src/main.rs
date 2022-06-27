use miarh::conf::Conf;

fn main() {
    let conf = Conf::new();
    println!("Here we go.\n{conf:?}");
}
