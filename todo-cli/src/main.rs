use std::fs;
use std::io::{self, Write};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: usize,
    description: String,
    done: bool,
}

impl Task {
    fn new(id: usize, description: String) -> Self {
        Self {
            id,
            description,
            done: false,
        }
    }
}

#[derive(Debug)]
struct TodoList {
    tasks: Vec<Task>,
}

impl TodoList {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    // Add a new task
    fn add_task(&mut self, description: String) {
        let id = self.tasks.len() + 1;
        let task = Task::new(id, description);
        self.tasks.push(task);
        println!("Task added successfully!");
    }

    // List all tasks
    fn list_tasks(&self) {
        if self.tasks.is_empty() {
            println!("No tasks found!");
        } else {
            for task in &self.tasks {
                let status = if task.done { "✓" } else { "✗" };
                println!("{}: [{}] {}", task.id, status, task.description);
            }
        }
    }

    // Mark a task as done
    fn mark_done(&mut self, id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.done = true;
            println!("Task {} marked as done!", id);
        } else {
            println!("Task not found!");
        }
    }

    // Save tasks to a file
    fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let data = serde_json::to_string(&self.tasks)?;
        fs::write(file_path, data)?;
        Ok(())
    }

    // Load tasks from a file
    fn load_from_file(file_path: &str) -> io::Result<Self> {
        let data = fs::read_to_string(file_path)?;
        let tasks: Vec<Task> = serde_json::from_str(&data)?;
        Ok(Self { tasks })
    }
}

fn main() {
    // Shared task manager using Arc<Mutex<TodoList>>
    let todo_list = Arc::new(Mutex::new(TodoList::new()));
    let file_path = "todo.json";

    // Load tasks from file if it exists
    if let Ok(loaded_list) = TodoList::load_from_file(file_path) {
        let mut list = todo_list.lock().unwrap();
        list.tasks = loaded_list.tasks;
    } else {
        println!("No existing to-do list found. Starting fresh.");
    }

    loop {
        println!("\nTo-Do List");
        println!("1. Add a task");
        println!("2. List all tasks");
        println!("3. Mark a task as done");
        println!("4. Simulate concurrent actions");
        println!("5. Save and exit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                print!("Enter task description: ");
                io::stdout().flush().unwrap();
                let mut description = String::new();
                io::stdin().read_line(&mut description).unwrap();
                let todo_list = Arc::clone(&todo_list);
                let description = description.trim().to_string();
                thread::spawn(move || {
                    let mut list = todo_list.lock().unwrap();
                    list.add_task(description);
                })
                .join()
                .unwrap();
            }
            "2" => {
                let list = todo_list.lock().unwrap();
                list.list_tasks();
            }
            "3" => {
                print!("Enter task ID to mark as done: ");
                io::stdout().flush().unwrap();
                let mut id = String::new();
                io::stdin().read_line(&mut id).unwrap();
                if let Ok(id) = id.trim().parse::<usize>() {
                    let todo_list = Arc::clone(&todo_list);
                    thread::spawn(move || {
                        let mut list = todo_list.lock().unwrap();
                        list.mark_done(id);
                    })
                    .join()
                    .unwrap();
                } else {
                    println!("Invalid ID!");
                }
            }
            "4" => {
                // Simulate concurrent actions: Add and mark tasks
                let todo_list1 = Arc::clone(&todo_list);
                let todo_list2 = Arc::clone(&todo_list);

                let handle1 = thread::spawn(move || {
                    let mut list = todo_list1.lock().unwrap();
                    list.add_task("Task from thread 1".to_string());
                });

                let handle2 = thread::spawn(move || {
                    let mut list = todo_list2.lock().unwrap();
                    list.add_task("Task from thread 2".to_string());
                });

                handle1.join().unwrap();
                handle2.join().unwrap();

                println!("Concurrent actions completed!");
            }
            "5" => {
                let list = todo_list.lock().unwrap();
                list.save_to_file(file_path).unwrap();
                println!("Tasks saved. Goodbye!");
                break;
            }
            _ => println!("Invalid choice!"),
        }
    }
}