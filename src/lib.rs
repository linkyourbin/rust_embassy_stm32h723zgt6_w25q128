// src/lib.rs

//! W25Q128JV SPI Flash Driver Library for `embassy-stm32`
//! 适用于 `embassy-stm32` 的 W25Q128JV SPI Flash 驱动库
//!
//! This library provides basic operations for the Winbond W25Q128JV serial Flash memory.
//! 该库提供了对 Winbond W25Q128JV 串行 Flash 存储器的基本操作。
//!
//! # Usage / 使用方法
//!
//! 1. Configure the SPI peripheral and CS pin.
//! 2. Create a `W25q128jv` instance.
//! 3. Call `init()` to initialize the device.
//! 4. Use the provided API for read, write, erase, etc.
//!
//! 1. 配置好 SPI 外设和 CS 引脚。
//! 2. 创建 `W25q128jv` 实例。
//! 3. 调用 `init()` 初始化设备。
//! 4. 使用提供的 API 进行读取、写入、擦除等操作。
//!
//! ```no_run
//! // Example code snippet (see examples/basic_usage.rs for full example)
//! // 示例代码片段 (见 examples/basic_usage.rs 获取完整示例)
//! #![no_std]
//! #![no_main]
//! # use embassy_executor::Spawner;
//! # use embassy_stm32::{spi::{Config as SpiConfig, Spi}, gpio::{Output, Level, Speed}};
//! # use defmt::info;
//! # use panic_probe as _;
//! # use defmt_rtt as _;
//! #
//! use embassy_stm32_w25q128jv::{W25q128jv, JEDEC_MAN_ID, JEDEC_MEM_TYPE, JEDEC_CAPACITY};
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let p = embassy_stm32::init(Default::default());
//!
//!     // --- Configure SPI and CS / 配置 SPI 和 CS ---
//!     let mut spi_config = SpiConfig::default();
//!     spi_config.frequency = embassy_stm32::time::Hertz(1_000_000);
//!     spi_config.mode = embassy_stm32::spi::Mode {
//!         polarity: embassy_stm32::spi::Polarity::IdleLow,
//!         phase: embassy_stm32::spi::Phase::CaptureOnFirstTransition,
//!     };
//!     let spi = Spi::new(
//!         p.SPI5, p.PF7, p.PF9, p.PF8,
//!         // Provide correct DMA channels if using DMA, otherwise pass None or omit
//!         // 如果使用 DMA，请提供正确的通道，否则可以传递 None 或省略
//!         // p.DMA2_CH7, p.DMA2_CH2,
//!         spi_config,
//!     );
//!     let cs = Output::new(p.PF6, Level::High, Speed::High);
//!
//!     // --- Create driver instance and initialize / 创建驱动实例并初始化 ---
//!     let mut flash = W25q128jv::new(spi, cs);
//!     flash.init().await;
//!
//!     // --- Use the driver / 使用驱动 ---
//!     match flash.read_jedec_id().await {
//!         Ok((man_id, mem_type, capacity)) => {
//!             if man_id == JEDEC_MAN_ID && mem_type == JEDEC_MEM_TYPE && capacity == JEDEC_CAPACITY {
//!                 info!("✅ Device identified successfully");
//!                 // info!("✅ 设备识别成功");
//!             } else {
//!                 info!("❌ ID mismatch");
//!                 // info!("❌ ID不匹配");
//!             }
//!         }
//!         Err(e) => info!("Failed to read ID: {:?}", e),
//!         // Err(e) => info!("读取ID失败: {:?}", e),
//!     }
//!     // ... more operations ...
//!     // ... 更多操作 ...
//! }
//! ```
//!
//! # Important Notes / 重要事项
//!
//! * **Hardware Connection / 硬件连接**:
//!   Ensure `/WP (IO2)` and `/HOLD or /RESET (IO3)` pins are pulled high for standard SPI mode.
//!   确保 `/WP (IO2)` 和 `/HOLD or /RESET (IO3)` 引脚在标准 SPI 模式下被拉高。
//! * **Error Handling / 错误处理**:
//!   The driver returns `embassy_stm32::spi::Error`. The caller must handle these errors.
//!   驱动返回 `embassy_stm32::spi::Error`。调用者需要处理这些错误。
//! * **Asynchronous / 异步**:
//!   All operations are asynchronous (`async`).
//!   所有操作都是异步的 (`async`)。
//!

#![no_std] // Declare as a no_std library / 声明为 no_std 库

// Declare modules / 声明模块
mod w25q128jv;

// Re-export public items for easy access / 重新导出公共项，方便库使用者直接访问
pub use w25q128jv::{
    W25q128jv, // Driver struct / 驱动结构体
    JEDEC_MAN_ID, JEDEC_MEM_TYPE, JEDEC_CAPACITY, // Constants / 常量
    SECTOR_SIZE, // Constants / 常量
    // If there are other public functions or types, export them here too
    // 如果有其他公共函数或类型，也需要在这里导出
};
// If there's an error type in the future, it should also be exported
// 如果将来有错误类型，也应该导出
// pub use w25q128jv::Error;