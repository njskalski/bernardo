use log::LevelFilter;

mod io;
mod primitives;
mod view;
mod widget;
mod experiments;


fn main() {
    env_logger::builder()
        .filter(None, LevelFilter::Debug)
        .init();



    println!("Hello, world!");
}
