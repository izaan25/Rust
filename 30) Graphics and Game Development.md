# Graphics and Game Development in Rust

## Overview

Rust is increasingly popular for graphics and game development due to its performance, safety, and growing ecosystem. This guide covers graphics programming, game engines, rendering, and game development patterns in Rust.

---

## Graphics Ecosystem

### Core Graphics Crates

| Crate | Purpose | Features |
|-------|---------|----------|
| `wgpu` | Modern graphics abstraction | Vulkan, Metal, DirectX, OpenGL |
| `vulkano` | Vulkan bindings | Low-level Vulkan API |
| `bevy` | Game engine | ECS, rendering, physics |
| `ggez` | 2D game framework | Simple 2D games |
| `macroquad` | Game framework | Cross-platform 2D/3D |
| `egui` | GUI library | Immediate mode GUI |
| `winit` | Window management | Cross-platform windows |
| `glam` | Math library | Vectors, matrices, quaternions |

### Choosing the Right Tools

- **wgpu** - Modern, portable graphics abstraction
- **bevy** - Full-featured game engine with ECS
- **ggez** - Simple 2D game development
- **vulkano** - Low-level Vulkan programming
- **macroquad** - Rapid prototyping

---

## Basic Graphics with wgpu

### Setup

```toml
[dependencies]
wgpu = "0.16"
winit = "0.28"
pollster = "0.3"
bytemuck = "1.0"
cgmath = "0.18"
```

### Basic Renderer

```rust
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        
        // Instance
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        
        // Surface
        let surface = unsafe { instance.create_surface(window) };
        
        // Adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        // Device and Queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        
        // Surface Configuration
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        
        surface.configure(&device, &config);
        
        State {
            surface,
            device,
            queue,
            config,
            size,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }
        
        let command_buffer = encoder.finish();
        
        self.queue.submit(Some(command_buffer));
        output.present();
        
        Ok(())
    }
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    
    let mut state = State::new(&window).await;
    
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        state.resize(window.inner_size());
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.render().unwrap();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
```

---

## 2D Game Development with ggez

### Setup

```toml
[dependencies]
ggez = "0.9"
rand = "0.8"
```

### Simple 2D Game

```rust
use ggez::{Context, ContextBuilder, GameResult, event, graphics};
use ggez::event::{EventHandler, KeyCode};
use ggez::graphics::{Color, DrawParam, Font, Text};
use ggez::mint::Point2;

struct Player {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    size: f32,
}

impl Player {
    fn new() -> Self {
        Player {
            x: 400.0,
            y: 300.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            size: 20.0,
        }
    }
    
    fn update(&mut self) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;
        
        // Apply friction
        self.velocity_x *= 0.9;
        self.velocity_y *= 0.9;
        
        // Keep player on screen
        if self.x < self.size {
            self.x = self.size;
            self.velocity_x = 0.0;
        }
        if self.x > 800.0 - self.size {
            self.x = 800.0 - self.size;
            self.velocity_x = 0.0;
        }
        if self.y < self.size {
            self.y = self.size;
            self.velocity_y = 0.0;
        }
        if self.y > 600.0 - self.size {
            self.y = 600.0 - self.size;
            self.velocity_y = 0.0;
        }
    }
    
    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2::new(self.x, self.y),
            self.size,
            0.1,
            Color::BLUE,
        )?;
        
        graphics::draw(ctx, &circle, DrawParam::default())
    }
}

struct Enemy {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    size: f32,
    color: Color,
}

impl Enemy {
    fn new(x: f32, y: f32) -> Self {
        let mut rng = rand::thread_rng();
        Enemy {
            x,
            y,
            velocity_x: rng.gen_range(-2.0..2.0),
            velocity_y: rng.gen_range(-2.0..2.0),
            size: 15.0,
            color: Color::RED,
        }
    }
    
    fn update(&mut self) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;
        
        // Bounce off walls
        if self.x < self.size || self.x > 800.0 - self.size {
            self.velocity_x = -self.velocity_x;
        }
        if self.y < self.size || self.y > 600.0 - self.size {
            self.velocity_y = -self.velocity_y;
        }
    }
    
    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2::new(self.x, self.y),
            self.size,
            0.1,
            self.color,
        )?;
        
        graphics::draw(ctx, &circle, DrawParam::default())
    }
}

struct GameState {
    player: Player,
    enemies: Vec<Enemy>,
    score: u32,
    font: Font,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let font = Font::new(ctx, "/DejaVuSansMono.ttf")?;
        
        let mut enemies = Vec::new();
        for _ in 0..5 {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(50.0..750.0);
            let y = rng.gen_range(50.0..550.0);
            enemies.push(Enemy::new(x, y));
        }
        
        Ok(GameState {
            player: Player::new(),
            enemies,
            score: 0,
            font,
        })
    }
    
    fn check_collisions(&mut self) {
        let player_rect = graphics::Rect::new(
            self.player.x - self.player.size,
            self.player.y - self.player.size,
            self.player.size * 2.0,
            self.player.size * 2.0,
        );
        
        let mut enemies_to_remove = Vec::new();
        
        for (i, enemy) in self.enemies.iter().enumerate() {
            let enemy_rect = graphics::Rect::new(
                enemy.x - enemy.size,
                enemy.y - enemy.size,
                enemy.size * 2.0,
                enemy.size * 2.0,
            );
            
            if player_rect.overlaps(&enemy_rect) {
                enemies_to_remove.push(i);
                self.score += 10;
            }
        }
        
        // Remove collided enemies
        for &index in enemies_to_remove.iter().rev() {
            self.enemies.remove(index);
        }
        
        // Spawn new enemies
        while self.enemies.len() < 5 {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(50.0..750.0);
            let y = rng.gen_range(50.0..550.0);
            self.enemies.push(Enemy::new(x, y));
        }
    }
}

impl EventHandler<ggez::event::KeyEvent> for GameState {
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: event::KeyMods, _repeat: bool) -> GameResult {
        match keycode {
            KeyCode::Up | KeyCode::W => self.player.velocity_y -= 5.0,
            KeyCode::Down | KeyCode::S => self.player.velocity_y += 5.0,
            KeyCode::Left | KeyCode::A => self.player.velocity_x -= 5.0,
            KeyCode::Right | KeyCode::D => self.player.velocity_x += 5.0,
            KeyCode::Escape => event::quit(ctx),
            _ => {}
        }
        Ok(())
    }
}

impl EventHandler<ggez::event::MainEvents> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.player.update();
        
        for enemy in &mut self.enemies {
            enemy.update();
        }
        
        self.check_collisions();
        
        Ok(())
    }
}

impl EventHandler<ggez::event::DrawEvent> for GameState {
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);
        
        // Draw player
        self.player.draw(ctx)?;
        
        // Draw enemies
        for enemy in &self.enemies {
            enemy.draw(ctx)?;
        }
        
        // Draw score
        let score_text = Text::new(format!("Score: {}", self.score), &self.font, 32.0)?;
        graphics::draw(ctx, &score_text, DrawParam::default().dest(Point2::new(10.0, 10.0)))?;
        
        graphics::present(ctx)
    }
}

fn main() -> GameResult {
    let cb = ContextBuilder::new("game", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("Rust Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));
    
    let (ctx, event_loop) = cb.build()?;
    
    let state = GameState::new(&ctx)?;
    
    event::run(ctx, event_loop, state)
}
```

---

## 3D Graphics with Bevy

### Setup

```toml
[dependencies]
bevy = "0.11"
```

### Basic 3D Scene

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(rotate_cube)
        .run();
}

#[derive(Component)]
struct RotatingCube;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Create a cube mesh
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    
    // Create a material
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.5, 1.0),
        ..default()
    });
    
    // Spawn the cube
    commands.spawn()
        .insert_bundle(PbrBundle {
            mesh: cube,
            material,
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(RotatingCube);
    
    // Add a camera
    commands.spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 2.0, 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        });
    
    // Add a light
    commands.spawn()
        .insert_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 1.0, 1.0),
                illuminance: 10000.0,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -std::f32::consts::PI / 4.0)),
            ..default()
        });
}

fn rotate_cube(mut cubes: Query<&mut Transform, With<RotatingCube>>, time: Res<Time>) {
    for mut transform in cubes.iter_mut() {
        transform.rotate_y(time.delta_seconds() * 2.0);
        transform.rotate_x(time.delta_seconds());
    }
}
```

### Bevy ECS Game

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Enemy {
    speed: f32,
}

#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

#[derive(Component)]
struct Damage {
    amount: u32,
}

#[derive(Resource)]
struct Score {
    value: u32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .add_startup_system(setup)
        .add_system(player_movement)
        .add_system(enemy_movement)
        .add_system(collision_detection)
        .add_system(apply_damage)
        .add_system(update_score)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn player
    commands.spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.6, 1.0),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Player { speed: 200.0 })
        .insert(Health { current: 100, max: 100 });
    
    // Spawn enemies
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * 2.0 * std::f32::consts::PI;
        let x = angle.cos() * 200.0;
        let y = angle.sin() * 200.0;
        
        commands.spawn()
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.3, 0.3),
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            })
            .insert(Enemy { speed: 50.0 })
            .insert(Health { current: 50, max: 50 });
    }
    
    // Add camera
    commands.spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    for mut transform in player_query.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
            direction.x += 1.0;
        }
        
        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * 200.0 * time.delta_seconds();
        }
    }
}

fn enemy_movement(
    mut enemy_query: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut enemy_transform in enemy_query.iter_mut() {
            let direction = (player_transform.translation - enemy_transform.translation).normalize();
            enemy_transform.translation += direction * 50.0 * time.delta_seconds();
        }
    }
}

fn collision_detection(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single() {
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let distance = player_transform.translation.distance(enemy_transform.translation);
            
            if distance < 45.0 {
                commands.entity(enemy_entity).insert(Damage { amount: 10 });
            }
        }
    }
}

fn apply_damage(
    mut commands: Commands,
    mut damage_query: Query<(Entity, &mut Health, &Damage)>,
    mut removed_entities: RemovedComponents<Health>,
) {
    for (entity, mut health, damage) in damage_query.iter_mut() {
        health.current = health.current.saturating_sub(damage.amount);
        
        if health.current == 0 {
            commands.entity(entity).despawn();
        }
    }
    
    for entity in removed_entities.iter() {
        println!("Entity {} died", entity.entity);
    }
}

fn update_score(
    mut score: ResMut<Score>,
    removed_enemies: RemovedComponents<Health>,
    enemies: Query<&Health, With<Enemy>>,
) {
    for removed in removed_enemies.iter() {
        // Check if this was an enemy
        if enemies.get(removed.entity).is_err() {
            score.value += 10;
        }
    }
    
    println!("Score: {}", score.value);
}
```

---

## Custom Game Engine

### Basic Engine Structure

```rust
use std::collections::HashMap;
use std::time::Instant;

// Entity Component System
type Entity = u64;

trait Component: 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

struct World {
    entities: Vec<Entity>,
    next_entity: Entity,
    components: HashMap<String, HashMap<Entity, Box<dyn Component>>>,
}

impl World {
    fn new() -> Self {
        World {
            entities: Vec::new(),
            next_entity: 1,
            components: HashMap::new(),
        }
    }
    
    fn create_entity(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }
    
    fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_name = std::any::type_name::<T>();
        let components = self.components.entry(type_name.to_string()).or_insert_with(HashMap::new);
        components.insert(entity, Box::new(component));
    }
    
    fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get(type_name) {
            if let Some(component) = components.get(&entity) {
                component.as_any().downcast_ref()
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn get_component_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get_mut(type_name) {
            if let Some(component) = components.get_mut(&entity) {
                component.as_any_mut().downcast_mut()
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn query<T: Component + 'static>(&self) -> Vec<(Entity, &T)> {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get(type_name) {
            components.iter()
                .filter_map(|(entity, component)| {
                    component.as_any().downcast_ref().map(|c| (*entity, c))
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

// Example Components
#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct Health {
    current: u32,
    max: u32,
}

impl Component for Health {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Systems
trait System {
    fn update(&mut self, world: &mut World, delta_time: f32);
}

struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        let entities: Vec<Entity> = world.query::<Position>().iter().map(|(e, _)| *e).collect();
        
        for entity in entities {
            if let (Some(position), Some(velocity)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Velocity>(entity),
            ) {
                let mut pos = world.get_component_mut::<Position>(entity).unwrap();
                pos.x += velocity.x * delta_time;
                pos.y += velocity.y * delta_time;
            }
        }
    }
}

struct HealthSystem;

impl System for HealthSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        let entities: Vec<Entity> = world.query::<Health>().iter().map(|(e, _)| *e).collect();
        
        for entity in entities {
            if let Some(health) = world.get_component::<Health>(entity) {
                if health.current == 0 {
                    println!("Entity {} died", entity);
                    // In a real engine, you'd remove the entity
                }
            }
        }
    }
}

// Game Loop
struct Game {
    world: World,
    systems: Vec<Box<dyn System>>,
    last_time: Instant,
}

impl Game {
    fn new() -> Self {
        let mut world = World::new();
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        
        // Create player
        let player = world.create_entity();
        world.add_component(player, Position { x: 0.0, y: 0.0 });
        world.add_component(player, Velocity { x: 100.0, y: 50.0 });
        world.add_component(player, Health { current: 100, max: 100 });
        
        // Create enemy
        let enemy = world.create_entity();
        world.add_component(enemy, Position { x: 200.0, y: 0.0 });
        world.add_component(enemy, Velocity { x: -50.0, y: 25.0 });
        world.add_component(enemy, Health { current: 50, max: 50 });
        
        // Add systems
        systems.push(Box::new(MovementSystem));
        systems.push(Box::new(HealthSystem));
        
        Game {
            world,
            systems,
            last_time: Instant::now(),
        }
    }
    
    fn run(&mut self) {
        let mut running = true;
        let mut frame_count = 0;
        
        while running && frame_count < 1000 {
            let now = Instant::now();
            let delta_time = now.duration_since(self.last_time).as_secs_f32();
            self.last_time = now;
            
            // Update systems
            for system in &mut self.systems {
                system.update(&mut self.world, delta_time);
            }
            
            // Print state every 60 frames
            if frame_count % 60 == 0 {
                println!("Frame {}: ", frame_count);
                for (entity, position) in self.world.query::<Position>() {
                    if let Some(health) = self.world.get_component::<Health>(*entity) {
                        println!("  Entity {}: pos=({:.1}, {:.1}), health={}/{}", 
                                entity, position.x, position.y, health.current, health.max);
                    }
                }
            }
            
            frame_count += 1;
            
            // Simple frame rate limiting
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

fn main() {
    let mut game = Game::new();
    game.run();
}
```

---

## Math and Physics

### Vector Math with glam

```toml
[dependencies]
glam = "0.24"
```

```rust
use glam::{Vec2, Vec3, Mat4, Quat};

fn vector_operations() {
    let v1 = Vec2::new(1.0, 2.0);
    let v2 = Vec2::new(3.0, 4.0);
    
    // Basic operations
    let sum = v1 + v2;
    let diff = v1 - v2;
    let dot = v1.dot(v2);
    let cross = v1.cross(v2);
    
    println!("Sum: {:?}", sum);
    println!("Difference: {:?}", diff);
    println!("Dot product: {}", dot);
    println!("Cross product: {:?}", cross);
    
    // 3D vectors
    let v3_1 = Vec3::new(1.0, 2.0, 3.0);
    let v3_2 = Vec3::new(4.0, 5.0, 6.0);
    
    let length = v3_1.length();
    let normalized = v3_1.normalize();
    
    println!("3D length: {}", length);
    println!("Normalized: {:?}", normalized);
}

fn matrix_operations() {
    // Translation matrix
    let translation = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    
    // Rotation matrix
    let rotation = Mat4::from_quat(Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, std::f32::consts::PI / 4.0));
    
    // Scale matrix
    let scale = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
    
    // Combined transformation
    let transform = translation * rotation * scale;
    
    println!("Transform matrix: {:?}", transform);
    
    // Transform a point
    let point = Vec3::new(1.0, 0.0, 0.0);
    let transformed = transform.transform_point3(point);
    
    println!("Transformed point: {:?}", transformed);
}

fn physics_simulation() {
    // Simple physics simulation
    struct RigidBody {
        position: Vec3,
        velocity: Vec3,
        acceleration: Vec3,
        mass: f32,
    }
    
    impl RigidBody {
        fn new(position: Vec3, mass: f32) -> Self {
            RigidBody {
                position,
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
                mass,
            }
        }
        
        fn apply_force(&mut self, force: Vec3) {
            self.acceleration += force / self.mass;
        }
        
        fn update(&mut self, dt: f32) {
            self.velocity += self.acceleration * dt;
            self.position += self.velocity * dt;
            self.acceleration = Vec3::ZERO; // Reset acceleration
        }
    }
    
    let mut body = RigidBody::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
    
    // Apply gravity
    body.apply_force(Vec3::new(0.0, -9.81, 0.0));
    
    // Simulate for 1 second
    let dt = 0.016; // 60 FPS
    let steps = (1.0 / dt) as u32;
    
    for _ in 0..steps {
        body.update(dt);
    }
    
    println!("Position after 1 second: {:?}", body.position);
    println!("Velocity after 1 second: {:?}", body.velocity);
}
```

---

## Key Takeaways

- **wgpu** provides modern, portable graphics abstraction
- **bevy** offers a complete game engine with ECS
- **ggez** is great for simple 2D games
- **ECS architecture** provides flexible component-based design
- **Math libraries** like glam handle vectors and matrices
- **Physics simulation** requires proper integration
- **Performance** depends on efficient rendering and updates
- **Cross-platform** development is well-supported

---

## Graphics and Game Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **ECS Architecture** | Component-based entity system | Use Bevy or custom ECS |
| **Asset Management** | Efficient resource loading | Use asset pipelines and caching |
| **Frame Rate** | Maintain consistent frame rate | Use delta time in updates |
| **Memory Management** | Avoid allocations in hot paths | Use object pools and reuse |
| **Rendering Optimization** | Batch similar draw calls | Use instancing and batching |
| **Physics Integration** | Realistic physics simulation | Use physics engines or custom |
| **Input Handling** | Responsive input processing | Use event-driven input |
| **Audio** | Sound effects and music | Use audio libraries |
| **UI Systems** | In-game interfaces | Use immediate mode or retained GUI |
