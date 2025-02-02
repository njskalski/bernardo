fn main() {
        let greeting = create_greeting("World");
    display_message(&greeting);
}

    fn create_greeting(name: &str) -> String {
            format!("Hello, {}!", name)
}

fn display_message(message: &str) {
    
                println!("{}", message);
}
