# Tors
## Simple Todo app written in Rust with Gamification

## Todo
- [x] Functionality todo list
- [x] TUI
- [ ] GUI (gtk4 + libadwaita)
- [ ] Gamification (soon)
- [ ] Optional encryption

### Build and install:
```shell
git clone https://github.com/reticulis/tors.git
cd tors
cargo build --release
cargo install --path .
```

# Usage
- change field - key up/key down

### Task list
- new task - n
- delete task - d
- edit task - enter
- close app - esc

### Edit mode
- edit title - t
- edit description - e
- edit preferences - p
- save task - s
- back to task list - esc

### Preferences mode
- change value - e
- back to task edit - esc