// »ʚ aichat "write me an example rust command line program that has more than 100 lines"
// Creating a Rust command-line program with more than 100 lines ensures complexity and helps demonstrate various Rust features. Let's construct a simple command-line to-do list application. This application will allow users to add, list, and remove tasks. The code will include handling user input, using structs, and basic file operations to persist data.

// Btw, I never ran this program

use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
struct Task {
    id: usize,
    description: String,
    completed: bool,
}

impl Task {
    fn new(id: usize, description: String) -> Task {
        Task {
            id,
            description,
            completed: false,
        }
    }

    fn from_line(line: &str) -> Option<Task> {
        let parts: Vec<&str> = line.trim().split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        let id = parts[0].parse().ok()?;
        let description = parts[1].to_string();
        let completed = parts[2].parse().ok()?;
        Some(Task {
            id,
            description,
            completed,
        })
    }

    fn to_string(&self) -> String {
        format!("{},{},{}", self.id, self.description, self.completed)
    }
}

fn main() -> io::Result<()> {
    let mut tasks = load_tasks("tasks.txt")?;

    loop {
        println!("\n-- To-Do List --");
        println!("1. List tasks");
        println!("2. Add task");
        println!("3. Remove task");
        println!("4. Mark task as completed");
        println!("5. Exit");

        let choice = get_user_input("\nChoose an option: ")?;

        match choice.trim() {
            "1" => list_tasks(&tasks),
            "2" => add_task(&mut tasks),
            "3" => remove_task(&mut tasks),
            "4" => complete_task(&mut tasks),
            "5" => break,
            _ => println!("Invalid choice, please try again."),
        }
    }

    save_tasks("tasks.txt", &tasks)?;
    Ok(())
}

fn get_user_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn load_tasks<P: AsRef<Path>>(path: P) -> io::Result<Vec<Task>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    let reader = BufReader::new(file);
    let mut tasks = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(task) = Task::from_line(&line) {
                tasks.push(task);
            }
        }
    }

    Ok(tasks)
}

fn save_tasks<P: AsRef<Path>>(path: P, tasks: &[Task]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;

    for task in tasks {
        writeln!(file, "{}", task.to_string())?;
    }

    Ok(())
}

fn list_tasks(tasks: &[Task]) {
    println!("\n-- Current Tasks --");
    for task in tasks {
        println!(
            "{}. [{}] - {}",
            task.id,
            if task.completed { "x" } else { " " },
            task.description
        );
    }
}

fn add_task(tasks: &mut Vec<Task>) {
    let description = match get_user_input("Enter task description: ") {
        Ok(desc) => desc.trim().to_string(),
        Err(_) => {
            println!("Failed to read input.");
            return;
        }
    };

    let id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;
    tasks.push(Task::new(id, description));
    println!("Task added.");
}

fn remove_task(tasks: &mut Vec<Task>) {
    let id = match get_user_input("Enter task ID to remove: ") {
        Ok(id_str) => match id_str.trim().parse::<usize>() {
            Ok(id) => id,
            Err(_) => {
                println!("Invalid ID.");
                return;
            }
        },
        Err(_) => {
            println!("Failed to read input.");
            return;
        }
    };

    tasks.retain(|task| task.id != id);
    println!("Task removed.");
}

fn complete_task(tasks: &mut Vec<Task>) {
    let id = match get_user_input("Enter task ID to complete: ") {
        Ok(id_str) => match id_str.trim().parse::<usize>() {
            Ok(id) => id,
            Err(_) => {
                println!("Invalid ID.");
                return;
            }
        },
        Err(_) => {
            println!("Failed to read input.");
            return;
        }
    };

    for task in tasks.iter_mut() {
        if task.id == id {
            task.completed = true;
            println!("Task marked as completed.");
            return;
        }
    }

    println!("Task not found.");
}
