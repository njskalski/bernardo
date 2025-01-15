pub mod mock {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref ALPHABET: Vec<&'static str> = vec![
            "alfa", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india", "juliet", "kilo", "lima", "mike", "november",
            "oscar", "papa", "quebec", "romeo", "sierra", "tango", "uniform", "victor", "whiskey", "x-ray", "yankee", "zulu"
        ];
    }
}
