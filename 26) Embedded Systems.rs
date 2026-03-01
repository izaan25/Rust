// 26_embedded_systems.rs
// Comprehensive examples of embedded systems programming in Rust

// Note: This file demonstrates embedded concepts but cannot be compiled directly
// without proper embedded target configuration and hardware-specific HALs.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use cortex_m_rt::exception;
use cortex_m::asm;

// =========================================
// PANIC HANDLER
// =========================================

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In a real embedded system, you might want to:
    // 1. Log the panic information
    // 2. Flash an LED to indicate error
    // 3. Reset the system
    // 4. Enter an infinite loop for debugging
    
    loop {
        // Flash LED or other error indication
        asm::nop();
    }
}

// =========================================
// EXCEPTION HANDLERS
// =========================================

#[exception]
fn HardFault(_ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // Handle hard fault exceptions
    // This is called on severe errors like stack overflow
    loop {
        asm::nop();
    }
}

#[exception]
fn DefaultHandler(_irqn: i16) {
    // Handle default exceptions
    // This catches any unhandled interrupts
}

// =========================================
// HARDWARE ABSTRACTION LAYER (HAL) EXAMPLES
// =========================================

// Simulated GPIO types (in real code, these come from HAL crates)
mod gpio {
    use core::marker::PhantomData;
    
    pub trait OutputPin {
        type Error;
        
        fn set_low(&mut self) -> Result<(), Self::Error>;
        fn set_high(&mut self) -> Result<(), Self::Error>;
        fn toggle(&mut self) -> Result<(), Self::Error>;
    }
    
    pub trait InputPin {
        type Error;
        
        fn is_high(&self) -> Result<bool, Self::Error>;
        fn is_low(&self) -> Result<bool, Self::Error>;
    }
    
    // Simulated LED driver
    pub struct Led {
        pin: u8,
        state: bool,
    }
    
    impl Led {
        pub fn new(pin: u8) -> Self {
            Led { pin, state: false }
        }
    }
    
    impl OutputPin for Led {
        type Error = ();
        
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.state = false;
            println!("LED {} set low", self.pin);
            Ok(())
        }
        
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.state = true;
            println!("LED {} set high", self.pin);
            Ok(())
        }
        
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.state = !self.state;
            println!("LED {} toggled to {}", self.pin, self.state);
            Ok(())
        }
    }
    
    // Simulated button
    pub struct Button {
        pin: u8,
        pressed: bool,
    }
    
    impl Button {
        pub fn new(pin: u8) -> Self {
            Button { pin, pressed: false }
        }
        
        pub fn simulate_press(&mut self) {
            self.pressed = !self.pressed;
        }
    }
    
    impl InputPin for Button {
        type Error = ();
        
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.pressed)
        }
        
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(!self.pressed)
        }
    }
}

// =========================================
// TIMER AND DELAY EXAMPLES
// =========================================

mod timer {
    use cortex_m::peripheral::SYST;
    
    pub struct Delay {
        syst: SYST,
    }
    
    impl Delay {
        pub fn new(syst: SYST) -> Self {
            Delay { syst }
        }
        
        pub fn delay_us(&mut self, us: u32) {
            // Simplified delay implementation
            for _ in 0..us {
                cortex_m::asm::nop();
            }
        }
        
        pub fn delay_ms(&mut self, ms: u32) {
            for _ in 0..ms {
                self.delay_us(1000);
            }
        }
    }
    
    // Hardware timer
    pub struct HardwareTimer {
        counter: u32,
        period: u32,
    }
    
    impl HardwareTimer {
        pub fn new(period: u32) -> Self {
            HardwareTimer {
                counter: 0,
                period,
            }
        }
        
        pub fn tick(&mut self) -> bool {
            self.counter += 1;
            if self.counter >= self.period {
                self.counter = 0;
                true // Timer expired
            } else {
                false
            }
        }
        
        pub fn reset(&mut self) {
            self.counter = 0;
        }
    }
}

// =========================================
// COMMUNICATION PROTOCOLS
// =========================================

mod communication {
    use core::fmt::Write;
    
    // Simulated UART
    pub struct Uart {
        tx_buffer: [u8; 64],
        tx_index: usize,
        rx_buffer: [u8; 64],
        rx_index: usize,
    }
    
    impl Uart {
        pub fn new() -> Self {
            Uart {
                tx_buffer: [0; 64],
                tx_index: 0,
                rx_buffer: [0; 64],
                rx_index: 0,
            }
        }
        
        pub fn write_byte(&mut self, byte: u8) {
            if self.tx_index < self.tx_buffer.len() {
                self.tx_buffer[self.tx_index] = byte;
                self.tx_index += 1;
                println!("UART TX: 0x{:02X}", byte);
            }
        }
        
        pub fn read_byte(&mut self) -> Option<u8> {
            if self.rx_index < self.rx_buffer.len() {
                let byte = self.rx_buffer[self.rx_index];
                self.rx_index += 1;
                Some(byte)
            } else {
                None
            }
        }
        
        pub fn simulate_rx(&mut self, byte: u8) {
            if self.rx_index < self.rx_buffer.len() {
                self.rx_buffer[self.rx_index] = byte;
                self.rx_index += 1;
            }
        }
    }
    
    impl Write for Uart {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for byte in s.bytes() {
                self.write_byte(byte);
            }
            Ok(())
        }
    }
    
    // Simulated I2C
    pub struct I2C {
        address: u8,
        buffer: [u8; 32],
    }
    
    impl I2C {
        pub fn new(address: u8) -> Self {
            I2C {
                address,
                buffer: [0; 32],
            }
        }
        
        pub fn write(&mut self, data: &[u8]) -> Result<(), ()> {
            println!("I2C Write to 0x{:02X}: {:?}", self.address, data);
            Ok(())
        }
        
        pub fn read(&mut self, len: usize) -> Result<&[u8], ()> {
            println!("I2C Read from 0x{:02X}: {} bytes", self.address, len);
            Ok(&self.buffer[..len])
        }
    }
    
    // Simulated SPI
    pub struct SPI {
        cs_active: bool,
    }
    
    impl SPI {
        pub fn new() -> Self {
            SPI { cs_active: false }
        }
        
        pub fn transfer(&mut self, data: &[u8]) -> Result<Vec<u8>, ()> {
            println!("SPI Transfer: {:?}", data);
            // Echo back the data for simulation
            Ok(data.to_vec())
        }
        
        pub fn cs_low(&mut self) {
            self.cs_active = true;
            println!("SPI CS Low");
        }
        
        pub fn cs_high(&mut self) {
            self.cs_active = false;
            println!("SPI CS High");
        }
    }
}

// =========================================
// SENSOR INTERFACES
// =========================================

mod sensors {
    use super::communication::I2C;
    use super::timer::Delay;
    
    // Temperature sensor interface
    pub trait TemperatureSensor {
        type Error;
        
        fn read_temperature(&mut self) -> Result<f32, Self::Error>;
    }
    
    // Simulated I2C temperature sensor (like LM75)
    pub struct I2CTemperatureSensor<I2C> {
        i2c: I2C,
        address: u8,
    }
    
    impl<I2C> I2CTemperatureSensor<I2C> {
        pub fn new(i2c: I2C, address: u8) -> Self {
            I2CTemperatureSensor { i2c, address }
        }
    }
    
    impl<I2C> TemperatureSensor for I2CTemperatureSensor<I2C>
    where
        I2C: embedded_hal::blocking::i2c::WriteRead,
    {
        type Error = ();
        
        fn read_temperature(&mut self) -> Result<f32, Self::Error> {
            let mut buffer = [0u8; 2];
            
            // Read temperature register
            self.i2c.write_read(self.address, &[0x00], &mut buffer)?;
            
            // Convert 12-bit value to Celsius
            let raw_temp = ((buffer[0] as u16) << 8) | (buffer[1] as u16);
            let temp_c = (raw_temp as f32) / 256.0;
            
            Ok(temp_c)
        }
    }
    
    // Accelerometer interface
    pub trait Accelerometer {
        type Error;
        
        fn read_acceleration(&mut self) -> Result<(f32, f32, f32), Self::Error>;
    }
    
    // Simulated SPI accelerometer
    pub struct SPIAccelerometer<SPI> {
        spi: SPI,
    }
    
    impl<SPI> SPIAccelerometer<SPI> {
        pub fn new(spi: SPI) -> Self {
            SPIAccelerometer { spi }
        }
    }
    
    impl<SPI> Accelerometer for SPIAccelerometer<SPI>
    where
        SPI: embedded_hal::blocking::spi::Transfer<u8>,
    {
        type Error = ();
        
        fn read_acceleration(&mut self) -> Result<(f32, f32, f32), Self::Error> {
            self.spi.cs_low();
            
            // Read acceleration registers
            let tx_data = [0x29, 0x00, 0x2B, 0x00, 0x2D, 0x00];
            let rx_data = self.spi.transfer(&tx_data)?;
            
            self.spi.cs_high();
            
            // Parse 12-bit signed values
            let x_raw = ((rx_data[1] as i16) << 4) | ((rx_data[2] as i16) >> 4);
            let y_raw = ((rx_data[3] as i16) << 4) | ((rx_data[4] as i16) >> 4);
            let z_raw = ((rx_data[5] as i16) << 4) | ((rx_data[6] as i16) >> 4);
            
            // Convert to g-force (assuming 4mg/LSB sensitivity)
            let x = (x_raw as f32) * 0.004;
            let y = (y_raw as f32) * 0.004;
            let z = (z_raw as f32) * 0.004;
            
            Ok((x, y, z))
        }
    }
}

// =========================================
// MEMORY MANAGEMENT
// =========================================

mod memory {
    use core::alloc::{GlobalAlloc, Layout};
    use core::ptr::NonNull;
    
    // Simple bump allocator for embedded systems
    pub struct BumpAllocator {
        heap_start: usize,
        heap_end: usize,
        current: usize,
    }
    
    impl BumpAllocator {
        pub const fn new() -> Self {
            BumpAllocator {
                heap_start: 0,
                heap_end: 0,
                current: 0,
            }
        }
        
        pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
            self.heap_start = heap_start;
            self.heap_end = heap_start + heap_size;
            self.current = heap_start;
        }
        
        pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
            let size = layout.size();
            let align = layout.align();
            
            let start = self.current;
            let aligned_start = (start + align - 1) & !(align - 1);
            let end = aligned_start + size;
            
            if end <= self.heap_end {
                self.current = end;
                unsafe {
                    Some(NonNull::new_unchecked(aligned_start as *mut u8))
                }
            } else {
                None
            }
        }
        
        pub fn reset(&mut self) {
            self.current = self.heap_start;
        }
    }
    
    unsafe impl GlobalAlloc for BumpAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // Note: This would need to be mutable in real implementation
            // For demonstration purposes only
            ptr::null_mut()
        }
        
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
            // Bump allocator doesn't free individual allocations
        }
    }
    
    // Static memory pool
    pub struct MemoryPool<T, const N: usize> {
        pool: [Option<T>; N],
        used: [bool; N],
    }
    
    impl<T, const N: usize> MemoryPool<T, N> {
        pub const fn new() -> Self {
            MemoryPool {
                pool: [None; N],
                used: [false; N],
            }
        }
        
        pub fn alloc(&mut self, item: T) -> Option<usize> {
            for i in 0..N {
                if !self.used[i] {
                    self.pool[i] = Some(item);
                    self.used[i] = true;
                    return Some(i);
                }
            }
            None
        }
        
        pub fn free(&mut self, index: usize) -> Option<T> {
            if index < N && self.used[index] {
                self.used[index] = false;
                self.pool[index].take()
            } else {
                None
            }
        }
        
        pub fn get(&self, index: usize) -> Option<&T> {
            if index < N && self.used[index] {
                self.pool[index].as_ref()
            } else {
                None
            }
        }
        
        pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
            if index < N && self.used[index] {
                self.pool[index].as_mut()
            } else {
                None
            }
        }
    }
}

// =========================================
// INTERRUPT HANDLING
// =========================================

mod interrupts {
    use core::sync::atomic::{AtomicBool, Ordering};
    
    // Interrupt-safe flag
    pub static INTERRUPT_FLAG: AtomicBool = AtomicBool::new(false);
    
    // Simulated interrupt handler
    pub fn handle_timer_interrupt() {
        println!("Timer interrupt triggered!");
        INTERRUPT_FLAG.store(true, Ordering::Relaxed);
    }
    
    // Critical section macro
    macro_rules! critical_section {
        ($($code:tt)*) => {
            cortex_m::interrupt::free(|_| {
                $($code)*
            })
        };
    }
    
    // Interrupt-safe counter
    pub static INTERRUPT_COUNTER: cortex_m::interrupt::Mutex<u32> = cortex_m::interrupt::Mutex::new(0);
    
    pub fn increment_counter() {
        cortex_m::interrupt::free(|cs| {
            let mut counter = INTERRUPT_COUNTER.borrow(cs);
            *counter += 1;
        });
    }
    
    pub fn get_counter() -> u32 {
        cortex_m::interrupt::free(|cs| {
            *INTERRUPT_COUNTER.borrow(cs)
        })
    }
}

// =========================================
// POWER MANAGEMENT
// =========================================

mod power {
    use cortex_m::asm;
    
    pub enum SleepMode {
        Sleep,      // CPU sleep, peripherals active
        Stop,       // CPU and most peripherals stopped
        Standby,    // Almost everything off
    }
    
    pub fn enter_sleep_mode(mode: SleepMode) {
        match mode {
            SleepMode::Sleep => {
                println!("Entering sleep mode");
                // Configure wake-up sources
                // Enter sleep mode
                asm::wfi();
                println!("Woke from sleep mode");
            }
            SleepMode::Stop => {
                println!("Entering stop mode");
                // Configure wake-up sources
                // Enter stop mode
                asm::wfi();
                println!("Woke from stop mode");
            }
            SleepMode::Standby => {
                println!("Entering standby mode");
                // Configure wake-up sources
                // Enter standby mode
                asm::wfi();
                println!("Woke from standby mode");
            }
        }
    }
    
    pub fn configure_clocks() {
        println!("Configuring system clocks");
        // In real implementation:
        // - Configure PLL
        // - Set system clock frequency
        // - Enable/disable peripheral clocks
        // - Configure clock gating for power savings
    }
    
    pub fn enable_low_power_mode() {
        println!("Enabling low power mode");
        // In real implementation:
        // - Reduce clock frequency
        // - Enable voltage scaling
        // - Disable unused peripherals
    }
}

// =========================================
// APPLICATION EXAMPLES
// =========================================

// LED blinker application
mod led_blinker {
    use super::gpio::{Led, OutputPin};
    use super::timer::Delay;
    
    pub fn run() -> ! {
        println!("Starting LED blinker application");
        
        let mut led = Led::new(13); // PC13 on STM32F411
        let mut delay = Delay::new(unsafe { cortex_m::Peripherals::steal().SYST });
        
        loop {
            led.set_high().unwrap();
            delay.delay_ms(500);
            
            led.set_low().unwrap();
            delay.delay_ms(500);
        }
    }
}

// Button monitor application
mod button_monitor {
    use super::gpio::{Button, InputPin, Led, OutputPin};
    use super::interrupts::{critical_section, INTERRUPT_FLAG};
    
    pub fn run() -> ! {
        println!("Starting button monitor application");
        
        let mut button = Button::new(0);
        let mut led = Led::new(13);
        let mut last_state = false;
        
        loop {
            let current_state = button.is_high().unwrap();
            
            if current_state != last_state {
                if current_state {
                    led.set_high().unwrap();
                    println!("Button pressed, LED on");
                } else {
                    led.set_low().unwrap();
                    println!("Button released, LED off");
                }
                last_state = current_state;
            }
            
            // Simulate interrupt handling
            critical_section(|| {
                if INTERRUPT_FLAG.load(core::sync::atomic::Ordering::Relaxed) {
                    println!("Processing interrupt flag");
                    INTERRUPT_FLAG.store(false, core::sync::atomic::Ordering::Relaxed);
                }
            });
            
            // Small delay to prevent busy-waiting
            for _ in 0..1000 {
                cortex_m::asm::nop();
            }
        }
    }
}

// Sensor data logger
mod sensor_logger {
    use super::communication::Uart;
    use super::sensors::{TemperatureSensor, I2CTemperatureSensor};
    use super::timer::Delay;
    
    pub fn run() -> ! {
        println!("Starting sensor data logger application");
        
        let mut uart = Uart::new();
        let mut i2c = super::communication::I2C::new(0x48);
        let mut sensor = I2CTemperatureSensor::new(i2c, 0x48);
        let mut delay = Delay::new(unsafe { cortex_m::Peripherals::steal().SYST });
        
        loop {
            match sensor.read_temperature() {
                Ok(temp) => {
                    writeln!(uart, "Temperature: {:.2}°C", temp).unwrap();
                }
                Err(_) => {
                    writeln!(uart, "Error reading temperature").unwrap();
                }
            }
            
            delay.delay_ms(1000);
        }
    }
}

// =========================================
// MAIN ENTRY POINT
// =========================================

#[entry]
fn main() -> ! {
    println!("=== EMBEDDED SYSTEMS DEMONSTRATION ===");
    println!("Note: This is a simulation of embedded concepts");
    println!("Real embedded code requires proper target configuration");
    println!();
    
    // In a real embedded system, you would:
    // 1. Initialize hardware peripherals
    // 2. Configure clocks and power management
    // 3. Set up interrupt handlers
    // 4. Start the main application loop
    
    // For demonstration, we'll run a simple simulation
    run_simulation();
    
    // In a real embedded system, we'd never return from main
    loop {
        cortex_m::asm::nop();
    }
}

fn run_simulation() {
    // Simulate basic embedded operations
    
    // GPIO operations
    println!("=== GPIO OPERATIONS ===");
    let mut led = super::gpio::Led::new(13);
    led.set_high().unwrap();
    led.toggle().unwrap();
    led.set_low().unwrap();
    
    // Timer operations
    println!("\n=== TIMER OPERATIONS ===");
    let mut timer = super::timer::HardwareTimer::new(1000);
    for _ in 0..5 {
        let expired = timer.tick();
        println!("Timer tick: {}", if expired { "expired" } else { "running" });
    }
    
    // Communication operations
    println!("\n=== COMMUNICATION OPERATIONS ===");
    let mut uart = super::communication::Uart::new();
    writeln!(uart, "Hello from embedded Rust!").unwrap();
    
    let mut i2c = super::communication::I2C::new(0x48);
    i2c.write(&[0x01, 0x02]).unwrap();
    
    let mut spi = super::communication::SPI::new();
    spi.cs_low();
    let _ = spi.transfer(&[0x9F, 0x00]);
    spi.cs_high();
    
    // Memory management
    println!("\n=== MEMORY MANAGEMENT ===");
    let mut pool: super::memory::MemoryPool<i32, 8> = super::memory::MemoryPool::new();
    let index1 = pool.alloc(42).unwrap();
    let index2 = pool.alloc(100).unwrap();
    
    println!("Allocated values: {} at index {}, {} at index {}", 
             pool.get(index1).unwrap(), index1,
             pool.get(index2).unwrap(), index2);
    
    // Interrupt handling
    println!("\n=== INTERRUPT HANDLING ===");
    super::interrupts::increment_counter();
    super::interrupts::increment_counter();
    println!("Interrupt counter: {}", super::interrupts::get_counter());
    
    // Power management
    println!("\n=== POWER MANAGEMENT ===");
    super::power::configure_clocks();
    super::power::enable_low_power_mode();
    
    println!("\n=== SIMULATION COMPLETE ===");
}

// =========================================
// UNIT TESTS (for simulation only)
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_led_operations() {
        let mut led = gpio::Led::new(13);
        assert!(led.set_high().is_ok());
        assert!(led.toggle().is_ok());
        assert!(led.set_low().is_ok());
    }
    
    #[test]
    fn test_timer() {
        let mut timer = timer::HardwareTimer::new(10);
        for i in 0..10 {
            let expired = timer.tick();
            assert_eq!(expired, i == 9);
        }
    }
    
    #[test]
    fn test_uart() {
        let mut uart = communication::Uart::new();
        assert!(uart.write_byte(0x41).is_ok());
        assert!(uart.write_byte(0x42).is_ok());
        assert_eq!(uart.read_byte(), Some(0x41));
        assert_eq!(uart.read_byte(), Some(0x42));
    }
    
    #[test]
    fn test_memory_pool() {
        let mut pool: memory::MemoryPool<i32, 4> = memory::MemoryPool::new();
        
        let index1 = pool.alloc(10);
        let index2 = pool.alloc(20);
        let index3 = pool.alloc(30);
        let index4 = pool.alloc(40);
        let index5 = pool.alloc(50); // Should fail
        
        assert!(index1.is_some());
        assert!(index2.is_some());
        assert!(index3.is_some());
        assert!(index4.is_some());
        assert!(index5.is_none());
        
        assert_eq!(pool.get(index1.unwrap()), Some(&10));
        assert_eq!(pool.get(index2.unwrap()), Some(&20));
        
        let freed = pool.free(index1.unwrap());
        assert_eq!(freed, Some(10));
        assert_eq!(pool.get(index1.unwrap()), None);
    }
    
    #[test]
    fn test_interrupt_counter() {
        let initial = interrupts::get_counter();
        interrupts::increment_counter();
        interrupts::increment_counter();
        assert_eq!(interrupts::get_counter(), initial + 2);
    }
}
