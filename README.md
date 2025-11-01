# TerminalDropDown

A terminal-based interactive dropdown selection component for Rust, enabling users to navigate options with keyboard arrows, confirm selections with Enter, and cancel with Escape.

## Overview

`TerminalDropDown` provides an interactive terminal interface for selecting options from a list. It leverages raw terminal mode (via `crossterm`) for low-level input handling and offers visual feedback with highlighted selected items, making it ideal for CLI applications requiring user input selection.

## Features

- Interactive navigation using up/down arrow keys (with wrap-around behavior for first/last items)
- Visual highlighting of the currently selected item (reverse video using ANSI escape codes)
- Support for large lists with automatic scrolling window (sliding view when items exceed visible area)
- Clean terminal handling with proper raw mode management (auto-enable on start, auto-disable on exit)
- Callback functions triggered on item selection (with the selected item passed as a parameter)
- Thread-safe operation using `Arc` and `Mutex` for shared state management
- Clear user instructions and status information (total items, visible range)
- Robust error handling for terminal operations and input events

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
terminal_dropdown = "0.1.0"  # Replace with actual version
crossterm = "0.27.0"
```

## Usage Example

```rust
use std::collections::HashMap;
use terminal_dropdown::TerminalDropDown;

fn main() {
    // Create a HashMap of options and their callback functions
    let mut options: HashMap<&str, Box<dyn FnMut(&&str) + Send>> = HashMap::new();
    
    options.insert("File 1.txt", Box::new(|selected| {
        println!("Processing selected file: {}", selected);
        // Add custom logic for handling "File 1.txt"
    }));
    
    options.insert("File 2.txt", Box::new(|selected| {
        println!("Opening: {}", selected);
        // Add custom logic for handling "File 2.txt"
    }));
    
    options.insert("File 3.txt", Box::new(|selected| {
        println!("Deleting: {}", selected);
        // Add custom logic for handling "File 3.txt"
    }));
    
    // Create and start the dropdown with a maximum of 5 visible items
    let dropdown = TerminalDropDown::use_drop_down(options, 5);
    
    // Wait for user interaction to complete
    if let Err(e) = dropdown.wait() {
        eprintln!("Error during dropdown interaction: {}", e);
    }
}
```

# License
[MIT]()