use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;
use std::{
    io::{self, prelude::*},
    thread,
};

/// A terminal-based interactive dropdown selection component.
///
/// Allows users to navigate through options using keyboard arrows, confirm selections with Enter,
/// and cancel with Escape. Uses raw terminal mode for input handling and provides visual feedback
/// with highlighted selected items.
///
/// # Type Parameters
/// * `T` - The type of items in the dropdown. Must implement necessary traits for display, hashing,
///   cloning, thread safety, and equality checks.
/// * `F` - The type of callback function triggered when an item is selected. Takes a reference to `T`
///   as a parameter.
#[derive(Debug)]
pub struct TerminalDropDown<T, F>
where
    T: Display + Hash + Clone + Send + Eq + 'static,
    F: FnOnce(&T) + Send + 'static,
{
    drop_down: Arc<Mutex<HashMap<T, F>>>,
    handle: JoinHandle<()>,
    item_n: usize,
}

impl<T, F> TerminalDropDown<T, F>
where
    T: Display + Hash + Clone + Send + Eq + 'static,
    F: FnOnce(&T) + Send + 'static,
{
    /// Creates a new TerminalDropDown instance and starts the interaction thread.
    ///
    /// # Parameters
    /// * `drop_down` - A HashMap containing items as keys and their corresponding callback functions
    ///   as values.
    /// * `item_n` - Maximum number of items to display in the terminal at once.
    ///
    /// # Returns
    /// A new TerminalDropDown instance ready for user interaction.
    ///
    /// # Behavior
    /// Spawns a new thread that handles user input, maintains selection state, and updates the display.
    /// Enables raw terminal mode for low-level input handling and properly cleans up resources.
    pub fn use_drop_down(drop_down: HashMap<T, F>, item_n: usize) -> Self {
        let drop_down = Arc::new(Mutex::new(drop_down));
        let cloned = drop_down.clone();

        let handle = thread::spawn(move || {
            let options: Vec<T> = cloned.lock().unwrap().keys().cloned().collect();
            if options.is_empty() {
                println!("\nNo options available.");
                return;
            }

            // 处理可能的错误而不是忽略
            if let Err(e) = enable_raw_mode() {
                eprintln!("Failed to enable raw mode: {}", e);
                return;
            }

            let mut current_idx = 0;
            Self::display_menu(&options, current_idx, item_n);

            let mut last_time = Instant::now();
            loop {
                // 处理事件读取错误
                let event = match event::read() {
                    Ok(Event::Key(key_event)) => key_event,
                    Ok(_) => continue, // 忽略非键盘事件
                    Err(e) => {
                        eprintln!("Failed to read event: {}", e);
                        break;
                    }
                };

                if Instant::now().duration_since(last_time).as_millis() < 300 {
                    continue;
                }
                last_time = Instant::now();

                match event.code {
                    KeyCode::Up => {
                        current_idx = if current_idx == 0 {
                            options.len() - 1
                        } else {
                            current_idx - 1
                        };
                        Self::display_menu(&options, current_idx, item_n);
                    }
                    KeyCode::Down => {
                        current_idx = (current_idx + 1) % options.len();
                        Self::display_menu(&options, current_idx, item_n);
                    }
                    KeyCode::Enter => {
                        let selected_key = &options[current_idx];
                        println!("\nConfirm delete: {}", selected_key);
                        if let Some(func) = cloned.lock().unwrap().remove(selected_key) {
                            func(selected_key);
                        }
                        break;
                    }
                    KeyCode::Esc => {
                        println!("\nDelete canceled.");
                        break;
                    }
                    _ => {}
                }
            }

            // 处理可能的错误而不是忽略
            if let Err(e) = disable_raw_mode() {
                eprintln!("Failed to disable raw mode: {}", e);
            }
        });

        Self {
            drop_down,
            handle,
            item_n,
        }
    }

    /// Renders the current state of the dropdown menu in the terminal.
    ///
    /// # Parameters
    /// * `options` - Slice of all available items in the dropdown.
    /// * `current_idx` - Index of the currently selected item.
    /// * `max_show` - Maximum number of items to display at once.
    ///
    /// # Behavior
    /// Clears the terminal, displays a header with total/max items, renders visible items with
    /// highlighting for the selected item, and shows navigation instructions. Implements a sliding
    /// window for when there are more items than can be displayed at once.
    pub fn display_menu(options: &[T], current_idx: usize, max_show: usize) {
        // Clear screen and reset cursor position
        print!("\x1B[2J\x1B[1;1H");
        // 处理刷新错误
        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {}", e);
        }

        if options.is_empty() {
            println!("No options available.\nPress ESC to exit.");
            return;
        }

        let total = options.len();
        let start_idx = if total <= max_show {
            0
        } else {
            current_idx
                .saturating_sub(max_show / 2)
                .min(total - max_show)
        };
        let end_idx = (start_idx + max_show).min(total);

        println!("Please select.（ESC for canceling）:");
        println!(
            "Total: {} | Showing: {} - {}\n",
            total,
            start_idx + 1,
            end_idx
        );

        for (i, option) in options
            .iter()
            .enumerate()
            .skip(start_idx)
            .take(end_idx - start_idx)
        {
            if i == current_idx {
                println!("\x1B[7m> {}\x1B[0m", option);
            } else {
                println!("  {}", option);
            }
        }

        println!("\n↑: Up | ↓: Down | Enter: Confirm | ESC: Cancel");
    }

    /// Blocks until the user interaction thread completes.
    ///
    /// # Returns
    /// A Result indicating whether the thread joined successfully or encountered an error.
    ///
    /// # Usage
    /// Call this method after creating the TerminalDropDown to wait for user input completion.
    pub fn wait(self) -> thread::Result<()> {
        self.handle.join()
    }
}