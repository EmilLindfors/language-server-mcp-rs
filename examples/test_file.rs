// Example Rust file to test the language server MCP

use std::collections::HashMap;

struct User {
    name: String,
    age: u32,
    email: String,
}

impl User {
    fn new(name: String, age: u32, email: String) -> Self {
        User { name, age, email }
    }

    fn greet(&self) -> String {
        format!("Hello, I'm {}", self.name)
    }
}

fn main() {
    // Create a new user
    let user = User::new("Alice".to_string(), 30, "alice@example.com".to_string());

    // Test hover: hover over 'user' to see its type
    println!("{}", user.greet());

    // Test completion: type 'user.' to see available methods
    let message = user.greet();

    // Test diagnostics: uncomment this line to see an error
    // let x: i32 = "not a number";

    // Test goto definition: click on HashMap to go to its definition
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert("Alice".to_string(), 100);

    // Test references: find all usages of the 'user' variable
    if user.age > 18 {
        println!("{} is an adult", user.name);
    }
}

// Helper function to demonstrate more features
fn calculate_score(scores: &HashMap<String, i32>, name: &str) -> Option<i32> {
    scores.get(name).copied()
}
