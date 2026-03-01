// 30_graphics_and_game_development.rs
// Comprehensive examples of graphics and game development in Rust

// Note: This file demonstrates graphics and game concepts but requires proper
// graphics libraries and hardware support to run actual rendering operations

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::any::Any;

// =========================================
// MATH AND PHYSICS
// =========================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    
    pub fn zero() -> Self {
        Vec2 { x: 0.0, y: 0.0 }
    }
    
    pub fn one() -> Self {
        Vec2 { x: 1.0, y: 1.0 }
    }
    
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Vec2 { x: self.x / len, y: self.y / len }
        } else {
            Vec2::zero()
        }
    }
    
    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
    
    pub fn cross(&self, other: &Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }
    
    pub fn distance(&self, other: &Vec2) -> f32 {
        (*self - *other).length()
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    
    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    
    fn mul(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    
    pub fn zero() -> Self {
        Vec3 { x: 0.0, y: 0.0, z: 0.0 }
    }
    
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            Vec3::zero()
        }
    }
    
    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
    
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }
    
    pub fn red() -> Self {
        Color::rgb(1.0, 0.0, 0.0)
    }
    
    pub fn green() -> Self {
        Color::rgb(0.0, 1.0, 0.0)
    }
    
    pub fn blue() -> Self {
        Color::rgb(0.0, 0.0, 1.0)
    }
    
    pub fn white() -> Self {
        Color::rgb(1.0, 1.0, 1.0)
    }
    
    pub fn black() -> Self {
        Color::rgb(0.0, 0.0, 0.0)
    }
}

// =========================================
// ENTITY COMPONENT SYSTEM
// =========================================

pub type Entity = u64;

pub trait Component: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct World {
    entities: Vec<Entity>,
    next_entity: Entity,
    components: HashMap<String, HashMap<Entity, Box<dyn Component>>>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            next_entity: 1,
            components: HashMap::new(),
        }
    }
    
    pub fn create_entity(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }
    
    pub fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_name = std::any::type_name::<T>();
        let components = self.components.entry(type_name.to_string()).or_insert_with(HashMap::new);
        components.insert(entity, Box::new(component));
    }
    
    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
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
    
    pub fn get_component_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
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
    
    pub fn query<T: Component + 'static>(&self) -> Vec<(Entity, &T)> {
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
    
    pub fn query_mut<T: Component + 'static>(&mut self) -> Vec<(Entity, &mut T)> {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get_mut(type_name) {
            components.iter_mut()
                .filter_map(|(entity, component)| {
                    component.as_any_mut().downcast_mut().map(|c| (*entity, c))
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

// =========================================
// GAME COMPONENTS
// =========================================

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
    
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Component for Position {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Velocity { x, y }
    }
    
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Component for Velocity {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    pub fn new(max: u32) -> Self {
        Health { current: max, max }
    }
    
    pub fn take_damage(&mut self, amount: u32) -> bool {
        if self.current > 0 {
            self.current = self.current.saturating_sub(amount);
            self.current == 0
        } else {
            false
        }
    }
    
    pub fn heal(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }
}

impl Component for Health {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Renderable {
    pub color: Color,
    pub size: f32,
    pub shape: Shape,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Circle,
    Square,
    Triangle,
}

impl Renderable {
    pub fn new(color: Color, size: f32, shape: Shape) -> Self {
        Renderable { color, size, shape }
    }
}

impl Component for Renderable {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub speed: f32,
}

impl Component for Player {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub speed: f32,
    pub damage: u32,
}

impl Component for Enemy {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub radius: f32,
}

impl Collider {
    pub fn new(radius: f32) -> Self {
        Collider { radius }
    }
    
    pub fn check_collision(&self, pos1: &Position, other: &Collider, pos2: &Position) -> bool {
        let dist = pos1.to_vec2().distance(&pos2.to_vec2());
        dist < (self.radius + other.radius)
    }
}

impl Component for Collider {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// =========================================
// SYSTEMS
// =========================================

pub trait System {
    fn update(&mut self, world: &mut World, delta_time: f32);
}

pub struct MovementSystem;

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

pub struct CollisionSystem {
    pub events: Vec<CollisionEvent>,
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity1: Entity,
    pub entity2: Entity,
}

impl CollisionSystem {
    pub fn new() -> Self {
        CollisionSystem {
            events: Vec::new(),
        }
    }
    
    pub fn get_events(&mut self) -> Vec<CollisionEvent> {
        let events = self.events.clone();
        self.events.clear();
        events
    }
}

impl System for CollisionSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        let colliders: Vec<(Entity, &Collider, &Position)> = world.query::<(Collider, Position)>()
            .iter()
            .map(|(e, (collider, pos))| (*e, collider, pos))
            .collect();
        
        for i in 0..colliders.len() {
            for j in (i + 1)..colliders.len() {
                let (entity1, collider1, pos1) = colliders[i];
                let (entity2, collider2, pos2) = colliders[j];
                
                if collider1.check_collision(pos1, collider2, pos2) {
                    self.events.push(CollisionEvent {
                        entity1,
                        entity2,
                    });
                }
            }
        }
    }
}

pub struct DamageSystem;

impl System for DamageSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        // Handle enemy-player collisions
        let players: Vec<Entity> = world.query::<Player>().iter().map(|(e, _)| *e).collect();
        let enemies: Vec<Entity> = world.query::<Enemy>().iter().map(|(e, _)| *e).collect();
        
        for player_entity in players {
            for enemy_entity in enemies.iter() {
                // Check if enemy has collided with player
                // In a real system, you'd use collision events
                if let (Some(player_health), Some(enemy)) = (
                    world.get_component_mut::<Health>(player_entity),
                    world.get_component::<Enemy>(*enemy_entity),
                ) {
                    // Simulate collision damage
                    player_health.take_damage(enemy.damage);
                }
            }
        }
    }
}

pub struct RenderSystem {
    pub frame_count: u64,
}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem { frame_count: 0 }
    }
    
    pub fn render(&mut self, world: &World) {
        self.frame_count += 1;
        
        println!("=== Frame {} ===", self.frame_count);
        
        let renderables = world.query::<(Position, Renderable)>();
        
        for (entity, (position, renderable)) in renderables {
            println!("Entity {}: pos=({:.1}, {:.1}), color=({:.2}, {:.2}, {:.2}, {:.2}), size={:.1}, shape={:?}",
                    entity, position.x, position.y, 
                    renderable.color.r, renderable.color.g, renderable.color.b, renderable.color.a,
                    renderable.size, renderable.shape);
        }
        
        // Show health for entities that have it
        let health_entities = world.query::<(Position, Health)>();
        for (entity, (position, health)) in health_entities {
            println!("Entity {}: pos=({:.1}, {:.1}), health={}/{}", 
                    entity, position.x, position.y, health.current, health.max);
        }
    }
}

// =========================================
// INPUT HANDLING
// =========================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyCode {
    Up,
    Down,
    Left,
    Right,
    Space,
    Escape,
}

pub struct Input {
    pub keys_pressed: Vec<KeyCode>,
    pub keys_just_pressed: Vec<KeyCode>,
    pub keys_just_released: Vec<KeyCode>,
}

impl Input {
    pub fn new() -> Self {
        Input {
            keys_pressed: Vec::new(),
            keys_just_pressed: Vec::new(),
            keys_just_released: Vec::new(),
        }
    }
    
    pub fn press_key(&mut self, key: KeyCode) {
        if !self.keys_pressed.contains(&key) {
            self.keys_pressed.push(key);
            self.keys_just_pressed.push(key);
        }
    }
    
    pub fn release_key(&mut self, key: KeyCode) {
        if let Some(pos) = self.keys_pressed.iter().position(|&k| k == key) {
            self.keys_pressed.remove(pos);
            self.keys_just_released.push(key);
        }
    }
    
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }
    
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }
    
    pub fn clear_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
    }
}

pub struct InputSystem;

impl System for InputSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        // This would be connected to actual input in a real game
        // For demonstration, we'll simulate some input
    }
}

pub struct PlayerInputSystem;

impl PlayerInputSystem {
    pub fn handle_input(&mut self, world: &mut World, input: &Input) {
        let players: Vec<Entity> = world.query::<Player>().iter().map(|(e, _)| *e).collect();
        
        for player_entity in players {
            if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                let mut new_velocity = Vec2::zero();
                
                if input.is_key_pressed(KeyCode::Up) || input.is_key_pressed(KeyCode::Left) {
                    new_velocity.y -= 200.0;
                }
                if input.is_key_pressed(KeyCode::Down) || input.is_key_pressed(KeyCode::Right) {
                    new_velocity.y += 200.0;
                }
                if input.is_key_pressed(KeyCode::Left) {
                    new_velocity.x -= 200.0;
                }
                if input.is_key_pressed(KeyCode::Right) {
                    new_velocity.x += 200.0;
                }
                
                velocity.x = new_velocity.x;
                velocity.y = new_velocity.y;
            }
        }
    }
}

// =========================================
// GAME ENGINE
// =========================================

pub struct Game {
    world: World,
    systems: Vec<Box<dyn System>>,
    render_system: RenderSystem,
    input_system: PlayerInputSystem,
    last_time: Instant,
    running: bool,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        
        // Create player
        let player = world.create_entity();
        world.add_component(player, Position::new(400.0, 300.0));
        world.add_component(player, Velocity::new(0.0, 0.0));
        world.add_component(player, Health::new(100));
        world.add_component(player, Renderable::new(Color::blue(), 20.0, Shape::Circle));
        world.add_component(player, Player { speed: 200.0 });
        world.add_component(player, Collider::new(20.0));
        
        // Create enemies
        for i in 0..5 {
            let angle = (i as f32 / 5.0) * 2.0 * std::f32::consts::PI;
            let x = 400.0 + angle.cos() * 200.0;
            let y = 300.0 + angle.sin() * 200.0;
            
            let enemy = world.create_entity();
            world.add_component(enemy, Position::new(x, y));
            world.add_component(enemy, Velocity::new(
                (i as f32 - 2.5) * 50.0,
                (i as f32 % 3) * 30.0
            ));
            world.add_component(enemy, Health::new(50));
            world.add_component(enemy, Renderable::new(Color::red(), 15.0, Shape::Square));
            world.add_component(enemy, Enemy { speed: 50.0, damage: 10 });
            world.add_component(enemy, Collider::new(15.0));
        }
        
        // Add systems
        systems.push(Box::new(MovementSystem));
        systems.push(Box::new(CollisionSystem::new()));
        systems.push(Box::new(DamageSystem));
        
        Game {
            world,
            systems,
            render_system: RenderSystem::new(),
            input_system: PlayerInputSystem,
            last_time: Instant::now(),
            running: true,
        }
    }
    
    pub fn run(&mut self) {
        let mut frame_count = 0;
        
        while self.running && frame_count < 600 { // Run for 10 seconds at 60 FPS
            let now = Instant::now();
            let delta_time = now.duration_since(self.last_time).as_secs_f32();
            self.last_time = now;
            
            // Simulate input
            let mut input = Input::new();
            if frame_count < 120 {
                input.press_key(KeyCode::Right);
            } else if frame_count < 240 {
                input.press_key(KeyCode::Left);
            } else if frame_count < 360 {
                input.press_key(KeyCode::Up);
            } else {
                input.press_key(KeyCode::Down);
            }
            
            // Handle input
            self.input_system.handle_input(&mut self.world, &input);
            
            // Update systems
            for system in &mut self.systems {
                system.update(&mut self.world, delta_time);
            }
            
            // Handle collision events
            if let Some(collision_system) = self.systems.get_mut(1) {
                if let Some(events) = collision_system.as_any_mut().downcast_mut::<CollisionSystem>() {
                    let collision_events = events.get_events();
                    for event in collision_events {
                        println!("Collision between entities {} and {}", event.entity1, event.entity2);
                    }
                }
            }
            
            // Render
            if frame_count % 60 == 0 { // Render every second
                self.render_system.render(&self.world);
            }
            
            frame_count += 1;
            
            // Frame rate limiting
            std::thread::sleep(Duration::from_millis(16));
        }
        
        println!("Game ended after {} frames", frame_count);
    }
}

// =========================================
// PHYSICS SIMULATION
// =========================================

pub struct RigidBody {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub mass: f32,
    pub radius: f32,
}

impl RigidBody {
    pub fn new(position: Vec3, mass: f32, radius: f32) -> Self {
        RigidBody {
            position,
            velocity: Vec3::zero(),
            acceleration: Vec3::zero(),
            mass,
            radius,
        }
    }
    
    pub fn apply_force(&mut self, force: Vec3) {
        self.acceleration += force / self.mass;
    }
    
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse / self.mass;
    }
    
    pub fn update(&mut self, dt: f32) {
        self.velocity += self.acceleration * dt;
        self.position += self.velocity * dt;
        self.acceleration = Vec3::zero(); // Reset acceleration
    }
    
    pub fn check_collision(&self, other: &RigidBody) -> bool {
        let distance = (self.position - other.position).length();
        distance < (self.radius + other.radius)
    }
    
    pub fn resolve_collision(&mut self, other: &mut RigidBody) {
        // Simple elastic collision
        let normal = (other.position - self.position).normalize();
        let relative_velocity = self.velocity - other.velocity;
        let velocity_along_normal = relative_velocity.dot(&normal);
        
        if velocity_along_normal > 0.0 {
            return; // Objects moving apart
        }
        
        let restitution = 0.8; // Bounciness
        let impulse = 2.0 * velocity_along_normal / (1.0 / self.mass + 1.0 / other.mass);
        
        self.velocity -= impulse * normal / self.mass * restitution;
        other.velocity += impulse * normal / other.mass * restitution;
        
        // Separate objects
        let overlap = (self.radius + other.radius) - (self.position - other.position).length();
        let separation = normal * (overlap / 2.0);
        self.position -= separation;
        other.position += separation;
    }
}

pub struct PhysicsWorld {
    pub bodies: Vec<RigidBody>,
    pub gravity: Vec3,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }
    
    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.push(body);
    }
    
    pub fn update(&mut self, dt: f32) {
        // Apply gravity
        for body in &mut self.bodies {
            body.apply_force(self.gravity * body.mass);
        }
        
        // Update positions
        for body in &mut self.bodies {
            body.update(dt);
        }
        
        // Check collisions
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                if self.bodies[i].check_collision(&self.bodies[j]) {
                    self.bodies[i].resolve_collision(&mut self.bodies[j]);
                }
            }
        }
    }
}

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub fn demonstrate_math_operations() {
    println!("=== MATH OPERATIONS ===");
    
    let v1 = Vec2::new(3.0, 4.0);
    let v2 = Vec2::new(1.0, 2.0);
    
    println!("v1 = {:?}", v1);
    println!("v2 = {:?}", v2);
    println!("v1 + v2 = {:?}", v1 + v2);
    println!("v1 - v2 = {:?}", v1 - v2);
    println!("v1 * 2.0 = {:?}", v1 * 2.0);
    println!("v1.length() = {}", v1.length());
    println!("v1.normalize() = {:?}", v1.normalize());
    println!("v1.dot(v2) = {}", v1.dot(&v2));
    println!("v1.cross(v2) = {}", v1.cross(&v2));
    println!("v1.distance(v2) = {}", v1.distance(&v2));
    
    let v3_1 = Vec3::new(1.0, 2.0, 3.0);
    let v3_2 = Vec3::new(4.0, 5.0, 6.0);
    
    println!("v3_1 = {:?}", v3_1);
    println!("v3_2 = {:?}", v3_2);
    println!("v3_1.length() = {}", v3_1.length());
    println!("v3_1.normalize() = {:?}", v3_1.normalize());
    println!("v3_1.dot(v3_2) = {}", v3_1.dot(&v3_2));
    println!("v3_1.cross(v3_2) = {:?}", v3_1.cross(&v3_2));
    
    println!();
}

pub fn demonstrate_ecs() {
    println!("=== ENTITY COMPONENT SYSTEM ===");
    
    let mut world = World::new();
    
    // Create entities with different components
    let player = world.create_entity();
    world.add_component(player, Position::new(100.0, 100.0));
    world.add_component(player, Velocity::new(5.0, 3.0));
    world.add_component(player, Health::new(100));
    world.add_component(player, Player { speed: 200.0 });
    
    let enemy = world.create_entity();
    world.add_component(enemy, Position::new(200.0, 150.0));
    world.add_component(enemy, Velocity::new(-2.0, 1.0));
    world.add_component(enemy, Health::new(50));
    world.add_component(enemy, Enemy { speed: 50.0, damage: 10 });
    
    // Query components
    println!("Entities with Position:");
    for (entity, position) in world.query::<Position>() {
        println!("  Entity {}: pos=({:.1}, {:.1})", entity, position.x, position.y);
    }
    
    println!("Entities with Velocity:");
    for (entity, velocity) in world.query::<Velocity>() {
        println!("  Entity {}: vel=({:.1}, {:.1})", entity, velocity.x, velocity.y);
    }
    
    println!("Entities with Health:");
    for (entity, health) in world.query::<Health>() {
        println!("  Entity {}: health={}/{}", entity, health.current, health.max);
    }
    
    // Modify components
    if let Some(health) = world.get_component_mut::<Health>(enemy) {
        health.take_damage(20);
        println!("Enemy health after damage: {}/{}", health.current, health.max);
    }
    
    println!();
}

pub fn demonstrate_physics() {
    println!("=== PHYSICS SIMULATION ===");
    
    let mut physics_world = PhysicsWorld::new();
    
    // Create falling objects
    physics_world.add_body(RigidBody::new(
        Vec3::new(0.0, 10.0, 0.0),
        1.0,
        0.5
    ));
    
    physics_world.add_body(RigidBody::new(
        Vec3::new(2.0, 15.0, 0.0),
        2.0,
        0.7
    ));
    
    // Simulate for 2 seconds
    let dt = 0.016; // 60 FPS
    let steps = (2.0 / dt) as u32;
    
    for i in 0..steps {
        physics_world.update(dt);
        
        if i % 60 == 0 { // Print every second
            println!("Time {:.1}s:", i as f32 * dt);
            for (j, body) in physics_world.bodies.iter().enumerate() {
                println!("  Body {}: pos=({:.2}, {:.2}, {:.2}), vel=({:.2}, {:.2}, {:.2})",
                        j, body.position.x, body.position.y, body.position.z,
                        body.velocity.x, body.velocity.y, body.velocity.z);
            }
        }
    }
    
    println!();
}

pub fn demonstrate_game_engine() {
    println!("=== GAME ENGINE DEMONSTRATION ===");
    
    let mut game = Game::new();
    game.run();
    
    println!();
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

fn main() {
    println!("=== GRAPHICS AND GAME DEVELOPMENT DEMONSTRATIONS ===\n");
    
    demonstrate_math_operations();
    demonstrate_ecs();
    demonstrate_physics();
    demonstrate_game_engine();
    
    println!("=== GRAPHICS AND GAME DEVELOPMENT DEMONSTRATIONS COMPLETE ===");
    println!("Note: This uses simulated graphics and game components. Real implementations would use:");
    println!("- wgpu or vulkano for graphics rendering");
    println!("- bevy or ggez for game frameworks");
    println!("- winit for window management");
    println!("- glam or cgmath for math operations");
    println!("- rapier or physx for physics engines");
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_operations() {
        let v1 = Vec2::new(3.0, 4.0);
        let v2 = Vec2::new(1.0, 2.0);
        
        assert_eq!(v1 + v2, Vec2::new(4.0, 6.0));
        assert_eq!(v1 - v2, Vec2::new(2.0, 2.0));
        assert_eq!(v1 * 2.0, Vec2::new(6.0, 8.0));
        assert_eq!(v1.length(), 5.0);
        assert_eq!(v1.dot(&v2), 11.0);
        assert_eq!(v1.cross(&v2), 2.0);
        assert_eq!(v1.distance(&v2), (5.0_f32).sqrt());
    }
    
    #[test]
    fn test_ecs() {
        let mut world = World::new();
        
        let entity = world.create_entity();
        world.add_component(entity, Position::new(10.0, 20.0));
        world.add_component(entity, Velocity::new(5.0, 3.0));
        
        assert_eq!(world.get_component::<Position>(entity).unwrap().x, 10.0);
        assert_eq!(world.get_component::<Velocity>(entity).unwrap().x, 5.0);
        
        let positions = world.query::<Position>();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].1.x, 10.0);
    }
    
    #[test]
    fn test_health() {
        let mut health = Health::new(100);
        
        assert!(health.is_alive());
        assert!(!health.take_damage(150));
        assert!(!health.is_alive());
        
        health = Health::new(100);
        health.take_damage(30);
        assert_eq!(health.current, 70);
        assert!(health.is_alive());
        
        health.heal(20);
        assert_eq!(health.current, 90);
        assert!(health.is_alive());
    }
    
    #[test]
    fn test_collision() {
        let collider1 = Collider::new(10.0);
        let collider2 = Collider::new(5.0);
        
        let pos1 = Position::new(0.0, 0.0);
        let pos2 = Position::new(12.0, 0.0);
        let pos3 = Position::new(20.0, 0.0);
        
        assert!(collider1.check_collision(&pos1, &collider2, &pos2));
        assert!(!collider1.check_collision(&pos1, &collider2, &pos3));
    }
    
    #[test]
    fn test_rigid_body() {
        let mut body = RigidBody::new(Vec3::new(0.0, 10.0, 0.0), 1.0, 1.0);
        
        body.apply_force(Vec3::new(0.0, -9.81, 0.0));
        body.update(1.0);
        
        assert_eq!(body.velocity.y, -9.81);
        assert_eq!(body.position.y, 0.19); // 10.0 + (-9.81 * 1.0) + (-9.81 * 1.0 * 1.0 / 2)
    }
    
    #[test]
    fn test_input() {
        let mut input = Input::new();
        
        assert!(!input.is_key_pressed(KeyCode::Up));
        
        input.press_key(KeyCode::Up);
        assert!(input.is_key_pressed(KeyCode::Up));
        assert!(input.is_key_just_pressed(KeyCode::Up));
        
        input.clear_frame();
        assert!(input.is_key_pressed(KeyCode::Up));
        assert!(!input.is_key_just_pressed(KeyCode::Up));
        
        input.release_key(KeyCode::Up);
        assert!(!input.is_key_pressed(KeyCode::Up));
        assert!(input.is_key_just_released(KeyCode::Up));
        
        input.clear_frame();
        assert!(!input.is_key_just_released(KeyCode::Up));
    }
    
    #[test]
    fn test_colors() {
        let red = Color::red();
        let blue = Color::blue();
        let white = Color::white();
        let black = Color::black();
        
        assert_eq!(red, Color::rgb(1.0, 0.0, 0.0));
        assert_eq!(blue, Color::rgb(0.0, 0.0, 1.0));
        assert_eq!(white, Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(black, Color::rgb(0.0, 0.0, 0.0));
    }
}
