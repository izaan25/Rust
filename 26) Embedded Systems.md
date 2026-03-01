# Embedded Systems in Rust

## Overview

Rust is increasingly popular for embedded systems development due to its memory safety, performance, and zero-cost abstractions. This guide covers embedded programming concepts, bare-metal development, and real-world embedded patterns.

---

## Embedded Rust Ecosystem

### Key Components

- **`embedded-hal`** - Hardware Abstraction Layer
- **`cortex-m`** - ARM Cortex-M processor support
- **`svd2rust`** - Register generation from vendor files
- **`probe-rs`** - Debugging and flashing tool
- **`defmt`** - Efficient logging for embedded systems

### Target Triples

```
thumbv7m-none-eabi    # ARM Cortex-M3/M4
thumbv6m-none-eabi    # ARM Cortex-M0/M0+
thumbv8m.base-none-eabi # ARM Cortex-M23
riscv32imc-unknown-none-elf # RISC-V 32-bit
```

---

## Bare-Metal Programming

### No Standard Library

```rust
#![no_std]  // Disable standard library
#![no_main] // Disable standard main entry point

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

### Entry Point

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Use the cortex-m-rt crate for entry point
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Your embedded code here
    loop {}
}
```

---

## Hardware Abstraction Layer (HAL)

### Using Embedded HAL

```toml
[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
stm32f4xx-hal = { version = "0.15", features = ["stm32f411"] }
```

### Basic GPIO Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    gpio::{GpioExt, Output, PushPull},
    prelude::*,
    stm32,
};

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Configure clocks
    let rcc = dp.RCC.constrain();
    let _clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // Configure GPIO
    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    loop {
        led.set_high();
        delay(cortex_m::asm::delay(1_000_000));
        led.set_low();
        delay(cortex_m::asm::delay(1_000_000));
    }
}
```

---

## Memory Management

### Static Memory Allocation

```rust
use heapless::pool::{Pool, Node};

// Static memory pool
static POOL: Pool<Node<[u8; 32]>, 8> = Pool::new();

fn use_memory_pool() {
    let block = POOL.alloc().unwrap();
    // Use the block
    POOL.free(block);
}
```

### Stack vs Heap

```rust
// Stack allocation (preferred in embedded)
const BUFFER_SIZE: usize = 256;
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

// Heap allocation (use with caution)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    panic!("allocation failed");
}
```

---

## Interrupts

### Interrupt Handlers

```rust
use cortex_m_rt::exception;
use cortex_m_rt::interrupt;

#[exception]
fn HardFault(_ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // Handle hard fault
    loop {}
}

#[interrupt]
fn EXTI0() {
    // Handle external interrupt 0
    // Clear interrupt flag
    // Process interrupt
}
```

### NVIC Configuration

```rust
use cortex_m::peripheral::NVIC;

fn configure_nvic() {
    let mut nvic = cortex_m::Peripherals::take().unwrap().NVIC;
    
    // Enable interrupt
    unsafe {
        nvic.enable(stm32f4xx_hal::interrupt::EXTI0);
        nvic.set_priority(stm32f4xx_hal::interrupt::EXTI0, 1);
    }
}
```

---

## Timers and Delays

### Delay Implementation

```rust
use cortex_m::delay::Delay;
use stm32f4xx_hal::prelude::*;

fn create_delay() -> Delay {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
    
    Delay::new(cp.SYST, clocks)
}
```

### Hardware Timers

```rust
use stm32f4xx_hal::timer::Timer;

fn setup_timer() {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
    
    let mut timer = Timer::tim2(dp.TIM2, clocks, &mut rcc.apb1);
    timer.start(1.hz());
    
    loop {
        if timer.wait().is_ok() {
            // Timer expired
        }
    }
}
```

---

## Communication Protocols

### UART/Serial

```rust
use stm32f4xx_hal::serial::{config::Config, Serial};

fn setup_uart() {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
    
    let gpioa = dp.GPIOA.split();
    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7();
    
    let serial = Serial::usart2(
        dp.USART2,
        (tx, rx),
        Config::default().baudrate(9600.bps()),
        clocks,
        &mut rcc.apb1,
    );
}
```

### I2C

```rust
use stm32f4xx_hal::i2c::I2c;

fn setup_i2c() {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.syscfg(48.mhz()).freeze();
    
    let gpiob = dp.GPIOB.split();
    let scl = gpiob.pb6.into_alternate_af4().set_open_drain();
    let sda = gpiob.pb7.into_alternate_af4().set_open_drain();
    
    let i2c = I2c::i2c1(
        dp.I2C1,
        (scl, sda),
        100.khz(),
        clocks,
        &mut rcc.apb1,
    );
}
```

### SPI

```rust
use stm32f4xx_hal::spi::Spi;

fn setup_spi() {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
    
    let gpioa = dp.GPIOA.split();
    let sck = gpioa.pa5.into_alternate_af5();
    let miso = gpioa.pa6.into_alternate_af5();
    let mosi = gpioa.pa7.into_alternate_af5();
    
    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        embedded_hal::spi::MODE_0,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );
}
```

---

## ADC and DAC

### Analog to Digital Converter

```rust
use stm32f4xx_hal::adc::{Adc, config::{AdcConfig, SampleTime, Sequence}};

fn setup_adc() {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
    
    let gpioa = dp.GPIOA.split();
    let analog_pin = gpioa.pa0.into_analog();
    
    let adc_config = AdcConfig::default()
        .default_sample_time(SampleTime::Cycles_480);
    
    let mut adc = Adc::adc1(dp.ADC1, true, adc_config);
    
    let value: u16 = adc.read(&analog_pin).unwrap();
}
```

### Digital to Analog Converter

```rust
use stm32f4xx_hal::dac::{Dac, Channel};

fn setup_dac() {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    
    let gpioa = dp.GPIOA.split();
    let analog_pin = gpioa.pa4.into_analog();
    
    let mut dac = Dac::dac1(dp.DAC, &mut rcc.apb1);
    
    dac.set_channel(Channel::C1, 2048); // Mid-scale value
    dac.enable(Channel::C1);
}
```

---

## Real-Time Considerations

### Deterministic Timing

```rust
use cortex_m::asm;

fn critical_section() {
    cortex_m::interrupt::free(|_| {
        // Critical section code
        // Interrupts are disabled here
    });
}

fn nop_delay(cycles: u32) {
    for _ in 0..cycles {
        asm::nop();
    }
}
```

### Priority Management

```rust
use cortex_m::peripheral::NVIC;

fn set_interrupt_priority() {
    let mut nvic = cortex_m::Peripherals::take().unwrap().NVIC;
    
    unsafe {
        nvic.set_priority(stm32f4xx_hal::interrupt::TIM2, 1); // High priority
        nvic.set_priority(stm32f4xx_hal::interrupt::TIM3, 2); // Medium priority
        nvic.set_priority(stm32f4xx_hal::interrupt::UART1, 3); // Low priority
    }
}
```

---

## Device Drivers

### Custom Driver Pattern

```rust
use embedded_hal::digital::v2::OutputPin;

struct LedDriver<LED: OutputPin> {
    led: LED,
}

impl<LED> LedDriver<LED>
where
    LED: OutputPin,
{
    fn new(led: LED) -> Self {
        LedDriver { led }
    }
    
    fn on(&mut self) -> Result<(), LED::Error> {
        self.led.set_high()
    }
    
    fn off(&mut self) -> Result<(), LED::Error> {
        self.led.set_low()
    }
    
    fn toggle(&mut self) -> Result<(), LED::Error> {
        self.led.toggle()
    }
}
```

### Sensor Interface

```rust
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};

struct TemperatureSensor<CS: OutputPin, DQ: InputPin + OutputPin> {
    cs: CS,
    dq: DQ,
}

impl<CS, DQ> TemperatureSensor<CS, DQ>
where
    CS: OutputPin,
    DQ: InputPin + OutputPin,
{
    fn new(cs: CS, dq: DQ) -> Self {
        TemperatureSensor { cs, dq }
    }
    
    fn read_temperature<DELAY>(&mut self, delay: &mut DELAY) -> Result<f32, Error>
    where
        DELAY: DelayUs<u32>,
    {
        // CS low to start conversion
        self.cs.set_low()?;
        delay.delay_us(10);
        
        // Read temperature data
        let temp_data = self.read_register(Register::TEMP)?;
        
        // CS high to end conversion
        self.cs.set_high()?;
        
        Ok(convert_to_celsius(temp_data))
    }
}
```

---

## Power Management

### Sleep Modes

```rust
use cortex_m::asm;

fn enter_sleep_mode() {
    // Configure wake-up sources
    // Set sleep mode
    cortex_m::asm::wfi(); // Wait for interrupt
}

fn enter_deep_sleep() {
    // Configure deep sleep
    // Set power down flags
    cortex_m::asm::wfi(); // Wait for interrupt
}
```

### Clock Gating

```rust
fn configure_clocks() {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    
    // Disable unused peripherals
    rcc.apb1.enr().modify(|_, w| {
        w.tim2en().disabled()
         .tim3en().disabled()
         .tim4en().disabled()
    });
    
    // Enable low-power run mode
    rcc.cr.modify(|_, w| w.lprun().set_bit());
}
```

---

## Debugging and Logging

### Efficient Logging with defmt

```toml
[dependencies]
defmt = "0.3"
defmt-rtt = "0.3"
panic-probe = "0.3"
```

```rust
use defmt::{debug, info, warn, error};

#[defmt::timestamp]
fn timestamp() -> u64 {
    // Return timestamp for logging
    0
}

fn log_example() {
    info!("System started");
    debug!("Debug message");
    warn!("Warning message");
    error!("Error message");
}
```

### Semihosting

```rust
use cortex_m_semihosting::hio;

fn semihosting_print() {
    if let Ok(mut hstdout) = hio::hstdout() {
        writeln!(hstdout, "Hello from embedded!").ok();
    }
}
```

---

## Build Configuration

### Cargo.toml for Embedded

```toml
[package]
name = "embedded-project"
version = "0.1.0"
edition = "2021"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
stm32f4xx-hal = { version = "0.15", features = ["stm32f411"] }

[[bin]]
name = "embedded-project"
test = false
bench = false

[profile.dev]
opt-level = 1
debug = true

[profile.release]
opt-level = "s"
lto = true
debug = true
codegen-units = 1
panic = "abort"
```

### Memory Layout (linker script)

```ld
/* Memory layout for STM32F411 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Stack size configuration */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

---

## Key Takeaways

- **No standard library** - Use `#![no_std]` for bare-metal
- **Hardware Abstraction Layer** - Use embedded-hal for portability
- **Memory constraints** - Prefer stack allocation over heap
- **Interrupt handling** - Critical for real-time responsiveness
- **Power management** - Essential for battery-powered devices
- **Debugging challenges** - Use specialized embedded tools
- **Deterministic behavior** - Avoid dynamic allocation in critical paths

---

## Common Embedded Crates

| Crate | Purpose | Use Case |
|-------|---------|----------|
| `cortex-m` | ARM Cortex-M support | Processor-specific code |
| `stm32f4xx-hal` | STM32F4 HAL | STM32 microcontrollers |
| `embedded-hal` | Hardware abstraction | Portable embedded code |
| `defmt` | Efficient logging | Debug embedded systems |
| `heapless` | Heapless data structures | Memory-constrained systems |
| `probe-rs` | Debugging/flashing | Development workflow |
| `svd2rust` | Register generation | Peripheral access |
