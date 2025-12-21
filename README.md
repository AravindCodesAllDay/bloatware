# Bloatware

A fast-paced top-down shooter built with Bevy Engine, featuring a **modular, feature-based architecture**.

## �️ Project Structure

The project uses a **Feature-Based Directory Structure** for scalability and maintainability. Each major feature has its own folder containing a `mod.rs` and relevant sub-modules.

```text
src/
├── main.rs                 # Entry point, plugin registration
├── common/                 # Shared resources (Constants, Physics, Collision)
│   ├── mod.rs
│   ├── constants.rs        # Game configuration
│   └── collision.rs        # Physics/Collision logic
├── player/                 # Player feature
│   └── mod.rs              # Components, Systems, Plugin
├── enemy/                  # Enemy feature
│   └── mod.rs              # Enemy AI, Components, Plugin
├── world/                  # World generation
│   ├── mod.rs
│   └── chunk.rs            # Infinite chunk generation
├── projectile/             # Projectile logic
│   └── mod.rs
├── game/                   # Core game loop & state
│   └── mod.rs
├── ui/                     # User Interface
│   └── mod.rs
└── audio/                  # Audio management
    └── mod.rs
```

## 👩‍💻 Coding Standards

### Explicit Imports
We adhere to a **React/Node.js style** of explicit imports to ensure code clarity.
**Do not use glob imports** (e.g., `use crate::common::constants::*;`).

**Good:**
```rust
use crate::common::constants::{PLAYER_SPEED, PLAYER_SIZE};
use crate::player::Player;
use bevy::prelude::{App, Plugin, Res, Query};
```

**Bad:**
```rust
use crate::common::constants::*; // Avoid this
use bevy::prelude::*;            // Avoid this
```

---

## 🧩 How to Create a New Plugin

Follow this template to add a new feature (e.g., `PowerUp`) that matches the project's style.

### 1. Create the Directory Structure
Create a new folder in `src/` with a `mod.rs`.

```bash
mkdir src/powerup
touch src/powerup/mod.rs
```

### 2. Implement the Plugin (`src/powerup/mod.rs`)

```rust
// ============================================================================
// powerup/mod.rs
use bevy::prelude::{
    App, Commands, Component, Plugin, Query, Res, Resource, Time, Update, Vec3, With, Sprite,
};
use crate::game::{Game, GameState};
// Import other specific types you need
// use crate::player::Player; 

#[derive(Component)]
pub struct PowerUp {
    pub duration: f32,
}

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, powerup_system);
    }
}

fn powerup_system(
    time: Res<Time>,
    mut query: Query<(&mut PowerUp, &Transform)>,
    game: Res<Game>,
) {
    // 1. Check Game State
    if game.state != GameState::Playing {
        return;
    }

    // 2. Logic
    for (mut powerup, transform) in query.iter_mut() {
        powerup.duration -= time.delta_secs();
        // ...
    }
}
```

### 3. Register in `main.rs`

```rust
// 1. Declare Module
mod powerup;

// 2. Import Plugin
use powerup::PowerUpPlugin;

fn main() {
    App::new()
        // ...
        // 3. Add Plugin
        .add_plugins(PowerUpPlugin)
        .run();
}
```

## � Building & Running

```bash
# Debug Build
cargo run

# Release Build (Game stays fast!)
cargo run --release
```

## 🎮 Controls

- **WASD / Arrows**: Move
- **SPACE**: Shoot
- **SHIFT**: Dash
- **ESC**: Pause
