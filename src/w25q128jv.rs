// src/w25q128jv.rs

//! W25Q128JV SPI Flash Driver / W25Q128JV SPI 闪存驱动
//!
//! Based on `embassy-stm32` and `embedded-hal`.
//! 基于 `embassy-stm32` 和 `embedded-hal`。
//!
//! Implements basic operations for the Winbond W25Q128JV Flash chip.
//! 实现了对 Winbond W25Q128JV Flash 芯片的基本操作。
//!
//! **Hardware Requirements / 硬件要求**:
//! Ensure `/WP (IO2)` and `/HOLD or /RESET (IO3)` pins are pulled high
//! (e.g., with 10kΩ resistors to VCC) for standard SPI mode.
//! 确保 `/WP (IO2)` 和 `/HOLD or /RESET (IO3)` 引脚在标准 SPI 模式下被拉高
//! （例如，通过 10kΩ 电阻连接到 VCC）。

use embassy_stm32::{
    gpio::Output,
    spi::{self, Spi},
    mode,
};
use embassy_time::Timer;
use embedded_hal::spi::SpiBus;

// --- Public Constants / 公共常量 ---

/// W25Q128JV Expected JEDEC Manufacturer ID / W25Q128JV 预期的 JEDEC 制造商 ID
pub const JEDEC_MAN_ID: u8 = 0xEF;
/// W25Q128JV Expected JEDEC Memory Type ID / W25Q128JV 预期的 JEDEC 内存类型 ID
pub const JEDEC_MEM_TYPE: u8 = 0x40;
/// W25Q128JV Expected JEDEC Capacity ID / W25Q128JV 预期的 JEDEC 容量 ID
pub const JEDEC_CAPACITY: u8 = 0x18;
/// W25Q128JV Sector Size (4KB) / W25Q128JV 扇区大小 (4KB)
pub const SECTOR_SIZE: usize = 4096;
// 可以根据需要添加更多常量，例如页面大小、块大小等

// --- Command Definitions / 命令定义 ---
/// W25Q128JV Command Definitions (per Datasheet Section 8.1)
/// W25Q128JV 命令定义（依据数据手册第 8.1 节）
mod commands {
    pub const READ_ID: u8 = 0x9F;              // Read JEDEC ID / 读取JEDEC ID
    pub const READ_STATUS_REG_1: u8 = 0x05;    // Read Status Register 1 / 读取状态寄存器1
    pub const WRITE_ENABLE: u8 = 0x06;         // Write Enable (required before write/erase) / 写使能（写入/擦除前必需）
    pub const READ_DATA: u8 = 0x03;            // Standard Read / 标准读取
    pub const FAST_READ: u8 = 0x0B;            // Fast Read / 快速读取
    pub const PAGE_PROGRAM: u8 = 0x02;         // Page Program / 页面编程
    pub const SECTOR_ERASE: u8 = 0xD8;         // 4KB Sector Erase / 4KB 扇区擦除
    // 可以根据需要添加更多命令，例如芯片擦除、块擦除等
}

// --- Driver Struct / 驱动结构体 ---
/// W25Q128JV Driver Instance / W25Q128JV 驱动实例
///
/// Represents a connection to a W25Q128JV Flash chip via SPI.
/// 代表通过 SPI 连接到 W25Q128JV Flash 芯片的实例。
pub struct W25q128jv<'d, M: mode::Mode> {
    spi: Spi<'d, M>,
    cs: Output<'d>,
}

// --- Driver Implementation / 驱动实现 ---
impl<'d, M: mode::Mode> W25q128jv<'d, M> {
    /// Creates a new W25Q128JV driver instance.
    /// 创建一个新的 W25Q128JV 驱动实例。
    ///
    /// # Arguments / 参数
    /// * `spi`: A configured SPI instance. / 已配置好的 SPI 实例。
    /// * `cs`: A GPIO output pin for /CS. / 用于 /CS 的 GPIO 输出引脚。
    ///
    pub fn new(spi: Spi<'d, M>, cs: Output<'d>) -> Self {
        Self { spi, cs }
    }

    /// Initializes the device: ensures CS transitions from high to low (per Datasheet Section 4.1).
    /// 初始化设备：确保CS经历高->低跳变（依据数据手册第4.1节）。
    ///
    /// This step is often required for Flash chips to wake up or enter a known state.
    /// 这个步骤对于某些 Flash 芯片是必需的，用于唤醒或进入已知状态。
    pub async fn init(&mut self) {
        // Force CS high (deselected) / 强制CS为高电平（未选中状态）
        self.cs.set_high();
        Timer::after_micros(10).await; // Wait for stability / 等待稳定
        // Generate high->low transition to activate the device / 产生高->低跳变，激活设备
        self.cs.set_low();
        Timer::after_micros(10).await; // Wait tCHSL (Datasheet 9.5 AC Characteristics) / 等待 tCHSL (数据手册 9.5 AC Characteristics)
        self.cs.set_high();
        Timer::after_micros(10).await; // Wait tSHSL1/SHSL2 (Datasheet 9.5 AC Characteristics) / 等待 tSHSL1/SHSL2 (数据手册 9.5 AC Characteristics)
        // Note: Logging here might not be available in a library context.
        // 注意：实际库中可能不直接打印日志。
        // info!("Device initialized, CS pin activated");
    }

    // --- Private Helper Functions / 私有辅助函数 ---

    /// Sends a single-byte command with no data.
    /// 发送单字节命令（无数据）。
    async fn command(&mut self, cmd: u8) -> Result<(), spi::Error> {
        self.cs.set_low();
        let result = self.spi.write(&[cmd]);
        self.cs.set_high();
        result
    }

    /// Sends a command and reads a single-byte response.
    /// 发送命令并读取响应（1字节）。
    async fn command_read_byte(&mut self, cmd: u8) -> Result<u8, spi::Error> {
        self.cs.set_low();
        self.spi.write(&[cmd])?; // Send command / 发送命令
        let mut buf = [0u8; 1];
        self.spi.read(&mut buf)?; // Read response immediately / 紧接着读取响应
        self.cs.set_high(); // Complete instruction, raise CS / 指令完成，拉高 CS
        Ok(buf[0])
    }

    /// Waits for the device to become idle (BUSY bit = 0).
    /// 等待设备空闲 (BUSY 位 = 0)。
    async fn wait_idle(&mut self) -> Result<(), spi::Error> {
        while self.is_busy().await? {
            Timer::after_micros(100).await; // Periodic check to avoid blocking / 周期性检查，避免长时间阻塞
        }
        Ok(())
    }

    // --- Public API Functions / 公共 API 函数 ---

    /// Reads the JEDEC ID (per Datasheet Section 8.2.27).
    /// 读取JEDEC ID（依据数据手册第8.2.27节）。
    ///
    /// Returns (Manufacturer ID, Memory Type, Capacity).
    /// 返回 (制造商 ID, 内存类型, 容量)。
    pub async fn read_jedec_id(&mut self) -> Result<(u8, u8, u8), spi::Error> {
        self.cs.set_low();
        // Send READ_ID command (0x9F) / 发送READ_ID命令（0x9F）
        self.spi.write(&[commands::READ_ID])?;
        // Read 3-byte response (Manufacturer ID + Memory Type + Capacity) / 读取3字节响应（制造商ID + 内存类型 + 容量）
        let mut buf = [0u8; 3];
        self.spi.read(&mut buf)?; // Read immediately after command / 紧接着读取3字节ID
        self.cs.set_high(); // Complete instruction, raise CS / 指令完成，拉高 CS
        Ok((buf[0], buf[1], buf[2]))
    }

    /// Reads Status Register 1 (per Datasheet Section 7.1.1).
    /// 读取状态寄存器1（依据数据手册第7.1.1节）。
    pub async fn read_status_register(&mut self) -> Result<u8, spi::Error> {
        self.command_read_byte(commands::READ_STATUS_REG_1).await
    }

    /// Checks if the device is busy (BUSY bit in Status Register, per Datasheet Section 7.1.1).
    /// 检查设备是否忙（状态寄存器中的 BUSY 位，依据数据手册第7.1.1节）。
    pub async fn is_busy(&mut self) -> Result<bool, spi::Error> {
        let status = self.read_status_register().await?;
        Ok((status & 0x01) != 0) // BUSY=1 means busy / BUSY=1表示忙
    }

    /// Standard Read data (per Datasheet Section 8.2.6).
    /// 标准读取数据（依据数据手册第8.2.6节）。
    ///
    /// # Arguments / 参数
    /// * `address`: The 24-bit address to start reading from. / 开始读取的 24 位地址。
    /// * `buf`: The buffer to read data into. / 用于存储读取数据的缓冲区。
    pub async fn read_data(&mut self, address: u32, buf: &mut [u8]) -> Result<(), spi::Error> {
        self.wait_idle().await?; // Wait for device to be idle / 等待设备空闲

        let cmd = commands::READ_DATA;
        // Pack 24-bit address / 打包 24 位地址
        let addr_bytes = [
            ((address >> 16) & 0xFF) as u8, // A23-A16
            ((address >> 8) & 0xFF) as u8,  // A15-A8
            (address & 0xFF) as u8,         // A7-A0
        ];

        self.cs.set_low();
        // Send command + 24-bit address / 发送命令+24位地址
        self.spi.write(&[cmd, addr_bytes[0], addr_bytes[1], addr_bytes[2]])?;
        // Read data / 读取数据
        self.spi.read(buf)?;
        self.cs.set_high(); // Complete instruction, raise CS / 指令完成，拉高 CS
        Ok(())
    }

    /// Fast Read data with dummy cycles (per Datasheet Section 8.2.7).
    /// 快速读取数据（带虚拟周期，依据数据手册第8.2.7节）。
    ///
    /// # Arguments / 参数
    /// * `address`: The 24-bit address to start reading from. / 开始读取的 24 位地址。
    /// * `buf`: The buffer to read data into. / 用于存储读取数据的缓冲区。
    pub async fn fast_read(&mut self, address: u32, buf: &mut [u8]) -> Result<(), spi::Error> {
        self.wait_idle().await?; // Wait for device to be idle / 等待设备空闲

        let cmd = commands::FAST_READ;
        // Pack 24-bit address / 打包 24 位地址
        let addr_bytes = [
            ((address >> 16) & 0xFF) as u8, // A23-A16
            ((address >> 8) & 0xFF) as u8,  // A15-A8
            (address & 0xFF) as u8,         // A7-A0
        ];

        self.cs.set_low();
        // Send command + address + 1 dummy byte (8 clocks) / 发送命令+地址+1字节虚拟周期（8个时钟）
        self.spi.write(&[cmd, addr_bytes[0], addr_bytes[1], addr_bytes[2], 0x00])?;
        self.spi.read(buf)?; // Read data / 读取数据
        self.cs.set_high(); // Complete instruction, raise CS / 指令完成，拉高 CS
        Ok(())
    }

    /// Write data to a page (Page Program, per Datasheet Section 8.2.13).
    /// 向页面写入数据（页面编程，依据数据手册第8.2.13节）。
    ///
    /// **Note**: The target area must be erased (set to 0xFF) before writing.
    /// **注意**: 写入前目标地址区域必须已被擦除（为 0xFF）。
    /// Data length must not exceed page size (typically 256 bytes) and must not cross page boundaries.
    /// 数据长度不能超过页面大小（通常 256 字节），且不能跨页面写入。
    ///
    /// # Arguments / 参数
    /// * `address`: The 24-bit address to start writing to. Must be page-aligned. / 开始写入的 24 位地址。必须按页面对齐。
    /// * `data`: The data slice to write. / 要写入的数据切片。
    pub async fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), spi::Error> {
        // Optional: Add length check for page size (e.g., 256 bytes)
        // 可选：添加长度检查 (例如，不超过 256 字节)
        // if data.len() > 256 { return Err(spi::Error::Other); }

        self.wait_idle().await?; // Wait for device to be idle / 等待设备空闲
        self.command(commands::WRITE_ENABLE).await?; // Send Write Enable / 发送写使能
        let cmd = commands::PAGE_PROGRAM;
        // Pack 24-bit address / 打包 24 位地址
        let addr_bytes = [
            ((address >> 16) & 0xFF) as u8,
            ((address >> 8) & 0xFF) as u8,
            (address & 0xFF) as u8,
        ];
        self.cs.set_low();
        // Send command + address + data / 发送命令+地址+数据
        self.spi.write(&[cmd, addr_bytes[0], addr_bytes[1], addr_bytes[2]])?;
        self.spi.write(data)?; // Write data / 写入数据
        self.cs.set_high();
        self.wait_idle().await?; // Wait for write to complete / 等待写入完成
        Ok(())
    }

    /// Erase a 4KB sector (per Datasheet Section 8.2.15).
    /// 擦除一个 4KB 扇区（依据数据手册第8.2.15节）。
    ///
    /// **Note**: This operation sets all bits in the sector to 1 (0xFF).
    /// **注意**: 此操作会将扇区内的所有位设置为 1 (0xFF)。
    ///
    /// # Arguments / 参数
    /// * `sector_address`: The 24-bit address of the sector to erase. Must be 4KB-aligned. / 要擦除的扇区的 24 位地址。必须按 4KB 对齐。
    pub async fn erase_sector(&mut self, sector_address: u32) -> Result<(), spi::Error> {
        // Optional: Add alignment check for sector size (4KB)
        // 可选：添加地址对齐检查 (4KB)
        // if sector_address % SECTOR_SIZE as u32 != 0 { return Err(spi::Error::Other); }

        self.wait_idle().await?; // Wait for device to be idle / 等待设备空闲
        self.command(commands::WRITE_ENABLE).await?; // Send Write Enable / 发送写使能
        let cmd = commands::SECTOR_ERASE;
        // Pack 24-bit address / 打包 24 位地址
        let addr_bytes = [
            ((sector_address >> 16) & 0xFF) as u8,
            ((sector_address >> 8) & 0xFF) as u8,
            (sector_address & 0xFF) as u8,
        ];
        self.cs.set_low();
        // Send command + address / 发送命令+地址
        self.spi.write(&[cmd, addr_bytes[0], addr_bytes[1], addr_bytes[2]])?;
        self.cs.set_high();
        self.wait_idle().await?; // Wait for erase to complete / 等待擦除完成
        Ok(())
    }

    // 可以根据需要添加更多 API 函数，例如：
    // pub async fn read_unique_id(&mut self) -> Result<[u8; 8], spi::Error> { ... }
    // pub async fn chip_erase(&mut self) -> Result<(), spi::Error> { ... } // 注意：耗时很长
    // pub async fn block_erase_32k(&mut self, address: u32) -> Result<(), spi::Error> { ... }
    // pub async fn block_erase_64k(&mut self, address: u32) -> Result<(), spi::Error> { ... }
    // pub async fn deep_power_down(&mut self) -> Result<(), spi::Error> { ... }
    // pub async fn release_from_power_down(&mut self) -> Result<(), spi::Error> { ... }
}