# Bloatware

A fast-paced top-down shooter built with Bevy Engine, featuring modular plugin architecture for easy development and maintenance.

## 🎮 Game Overview

**Bloatware** is an arcade-style shooter where you navigate through a maze-like arena, shooting targets while avoiding collisions. The game features:

- **Movement**: WASD or Arrow Keys
- **Dash**: Shift (with cooldown)
- **Shoot**: Space (with cooldown)
- **Objective**: Hit as many targets as possible without touching them

### Gameplay Mechanics

- **Cooldown System**: Visual indicators show when abilities are ready
  - Red fill: Dash ability
  - Yellow fill: Shoot ability
- **Score Tracking**: Each target hit increases your score
- **Timer**: Track how long you survive
- **Game Over**: Touching a target ends the game

## 🏗️ Project Structure

The project uses a **modular plugin architecture** for clean separation of concerns and easy maintenance:

```
src/
├── main.rs           # Entry point, plugin registration
├── constants.rs      # Game constants and configuration
├── game.rs          # Core game state management
├── audio.rs         # Audio asset loading
├── level.rs         # Map/wall generation
├── player.rs        # Player movement, dash, shooting
├── projectile.rs    # Projectile behavior
├── target.rs        # Target spawning logic
├── ui.rs            # UI rendering and updates
└── collision.rs     # Collision detection systems
```

## 📦 Module Breakdown

### `main.rs`

- Application entry point
- Registers all game plugins
- Configures window settings

### `constants.rs`

- Centralized game configuration
- Speed values, sizes, cooldowns
- Color definitions
- Level map layout

### `game.rs` - GamePlugin

**Responsibilities:**

- Game state management (Menu, Playing, GameOver)
- Score and timer tracking
- State transition handling
- Camera setup

**Key Resources:**

```rust
Game {
    state: GameState,
    score: u32,
    timer: f32,
}
```

### `audio.rs` - AudioPlugin

**Responsibilities:**

- Loading audio assets at startup
- Provides audio handles for game events

**Assets:**

- `shoot.ogg` - Shooting sound
- `dash.ogg` - Dash sound
- `hit.ogg` - Collision/hit sound
- `step.ogg` - Footstep sound

### `level.rs` - LevelPlugin

**Responsibilities:**

- Spawning walls from ASCII map
- Providing valid spawn positions
- Map coordinate calculations

**Key Functions:**

- `spawn_walls()` - Generates level geometry
- `get_valid_spawn_positions()` - Returns valid spawn locations

### `player.rs` - PlayerPlugin

**Responsibilities:**

- Player movement and input handling
- Dash mechanic with interpolation
- Shooting projectiles
- Visual cooldown indicators
- Footstep audio timing

**Components:**

```rust
Player {
    last_direction: Vec3,
    shoot_cooldown: f32,
    dash_cooldown: f32,
    is_dashing: bool,
    dash_timer: f32,
    dash_direction: Vec3,
    dash_start_pos: Vec3,
    step_timer: f32,
}
```

**Key Systems:**

- `player_movement()` - Handles input and movement
- `dash_handler()` - Smooth dash interpolation
- `update_player_visuals()` - Cooldown indicator fills

### `projectile.rs` - ProjectilePlugin

**Responsibilities:**

- Projectile movement
- Distance tracking
- Auto-despawn at max range

**Components:**

```rust
Projectile {
    direction: Vec3,
    distance_traveled: f32,
}
```

### `target.rs` - TargetPlugin

**Responsibilities:**

- Target spawning at valid positions
- Maintaining minimum distance from player
- Cleanup on menu state

**Key Functions:**

- `spawn_target()` - Spawns target away from player

### `ui.rs` - UIPlugin

**Responsibilities:**

- Menu screen display
- Score and timer rendering
- Game over screen

**Display Formats:**

- Score counter
- Time: `MM:SS.MS`
- State-dependent messages

### `collision.rs` - CollisionPlugin

**Responsibilities:**

- Player-wall collision with resolution
- Projectile-wall collision
- Projectile-target collision (scoring)
- Player-target collision (game over)

**Key Systems:**

- `check_player_wall_collision()` - AABB with push-out
- `check_projectile_wall_collision()` - Despawn on hit
- `check_projectile_target_collision()` - Score and respawn
- `check_player_target_collision()` - Trigger game over

## 🚀 Getting Started

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Bevy Dependencies**: See [Bevy Setup](https://bevyengine.org/learn/book/getting-started/setup/)

### Building

```bash
# Debug build
cargo build

# Release build (recommended for gameplay)
cargo build --release
```

### Running

```bash
# Debug mode
cargo run

# Release mode (better performance)
cargo run --release
```

### Asset Setup

Place audio files in the `assets/` directory:

```
assets/
├── shoot.ogg
├── dash.ogg
├── hit.ogg
└── step.ogg
```

## 🛠️ Development Guide

### Adding New Features

The modular structure makes it easy to extend:

#### Adding a New Enemy Type

1. Create `src/enemy.rs`:

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, enemy_movement);
    }
}

fn enemy_movement(/* ... */) {
    // Implementation
}
```

2. Register in `main.rs`:

```rust
mod enemy;
use enemy::EnemyPlugin;

fn main() {
    App::new()
        // ...
        .add_plugins(EnemyPlugin)
        .run();
}
```

3. Add collision detection in `collision.rs`

#### Modifying Game Balance

Edit values in `constants.rs`:

```rust
pub const PLAYER_SPEED: f32 = 400.0;  // Faster movement
pub const DASH_COOLDOWN: f32 = 0.4;   // Shorter cooldown
```

#### Adding New Audio

1. Add asset handle in `audio.rs`:

```rust
pub struct GameAssets {
    // ...
    pub new_sound: Handle<AudioSource>,
}
```

2. Load in `load_assets()`:

```rust
new_sound: asset_server.load("new_sound.ogg"),
```

3. Use in any system:

```rust
commands.spawn(AudioPlayer(assets.new_sound.clone()));
```

### System Execution Order

Bevy runs systems in parallel when possible. Key ordering:

1. Input handling (`game.rs`)
2. Movement updates (`player.rs`, `projectile.rs`)
3. Collision checks (`collision.rs`)
4. Visual updates (`player.rs`, `ui.rs`)

### Debugging Tips

**Enable console logging:**

```rust
// Add to any system
println!("Player position: {:?}", transform.translation);
```

**Inspect entities:**

```rust
for entity in query.iter() {
    info!("Entity: {:?}", entity);
}
```

**Visual debugging:**

- Modify colors in `constants.rs` to highlight entities
- Adjust cooldown durations for testing

## 🎨 Customization

### Changing the Map

Edit the ASCII map in `constants.rs`:

```rust
pub const LEVEL_MAP: &str = "\
##########
#........#
#..###...#
#........#
##########";
```

- `#` = Wall
- `.` = Empty space

### Modifying Colors

All colors are in `constants.rs`:

```rust
pub const PLAYER_READY: Color = Color::srgb(0.9, 0.2, 0.2);
pub const PROJECTILE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
```

### Adjusting Difficulty

Change spawn distance and speeds:

```rust
pub const MIN_SPAWN_DISTANCE: f32 = 200.0;  // Harder
pub const PLAYER_SPEED: f32 = 250.0;        // Slower
```

## 📋 Plugin Dependencies

Each plugin depends on specific resources and components:

| Plugin           | Depends On        | Provides            |
| ---------------- | ----------------- | ------------------- |
| GamePlugin       | -                 | Game, Camera        |
| AudioPlugin      | -                 | GameAssets          |
| LevelPlugin      | constants         | Wall spawning       |
| PlayerPlugin     | GameAssets, Game  | Player, movement    |
| ProjectilePlugin | Game              | Projectile movement |
| TargetPlugin     | Player (optional) | Target spawning     |
| UIPlugin         | Game              | UI rendering        |
| CollisionPlugin  | All components    | Collision handling  |

## 🔧 Technical Details

### Coordinate System

- Origin (0, 0) at screen center
- Y-axis points up
- Map is centered automatically

### Collision Detection

- **AABB** (Axis-Aligned Bounding Box) for all collisions
- Push-out resolution for player-wall
- Immediate despawn for projectile hits

### Dash Mechanic

- Linear interpolation from start to end position
- Cancels on wall collision
- Direction locked at dash start

### Performance

- Entity-Component-System (ECS) architecture
- Parallel system execution
- Minimal allocations during gameplay

## 📝 Future Improvements

Potential enhancements:

- [ ] Multiple enemy types with AI
- [ ] Power-ups (speed boost, rapid fire)
- [ ] Score multiplier system
- [ ] Level progression
- [ ] High score persistence
- [ ] Particle effects for hits
- [ ] Screen shake on dash/hit
- [ ] Sound volume controls
- [ ] Pause menu

## 📄 License

This project structure is provided as-is for educational purposes.

## 🤝 Contributing

When adding features:

1. Create a new module file for significant features
2. Keep systems focused on single responsibilities
3. Use the plugin pattern for organization
4. Update constants.rs for configuration values
5. Document public functions and complex logic

## 🐛 Common Issues

**Game runs slowly:**

- Use `cargo run --release` for optimized builds
- Check GPU drivers are up to date

**Audio not playing:**

- Verify `.ogg` files are in `assets/` directory
- Check file names match exactly

**Collision issues:**

- Adjust collision sizes in `constants.rs`
- Check system execution order

## 📚 Resources

- [Bevy Engine](https://bevyengine.org/)
- [Bevy Cheat Book](https://bevy-cheatbook.github.io/)
- [Rust Book](https://doc.rust-lang.org/book/)

---

**Happy coding! 🎮**
