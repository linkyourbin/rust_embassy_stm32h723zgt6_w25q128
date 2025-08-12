# embassy-stm32-w25q128jv

A `no_std` Rust driver for the Winbond W25Q128JV SPI Flash memory, designed for use with the `embassy-stm32` ecosystem.
适用于 `embassy-stm32` 生态系统的 Winbond W25Q128JV SPI 闪存 `no_std` Rust 驱动。

## Features / 功能

* Read JEDEC ID / 读取 JEDEC ID
* Read Status Register / 读取状态寄存器
* Standard Read (`03h`) / 标准读取 (`03h`)
* Fast Read (`0Bh`) / 快速读取 (`0Bh`)
* Page Program (`02h`) / 页面编程 (`02h`)
* Sector Erase (4KB, `D8h`) / 扇区擦除 (4KB, `D8h`)
* Wait for idle/busy status / 等待空闲/忙碌状态
* Designed for asynchronous operation with `embassy-time`. / 专为与 `embassy-time` 异步操作设计。
* Includes English and Chinese inline comments and documentation. / 包含英文和中文内联注释及文档。

## Hardware Requirements / 硬件要求

* Ensure `/WP (IO2)` and `/HOLD or /RESET (IO3)` pins are pulled high (e.g., with 10kΩ resistors to VCC) for standard SPI mode.
  确保标准 SPI 模式下 `/WP (IO2)` 和 `/HOLD or /RESET (IO3)` 引脚被拉高（例如，通过 10kΩ 电阻连接到 VCC）。
  * As per the W25Q128JV datasheet (Section 4.3), the `/WP` pin is active low and can be used to prevent writing to the Status Register.
    根据 W25Q128JV 数据手册（第 4.3 节），`/WP` 引脚是低电平有效，可用于防止写入状态寄存器。
  * As per the W25Q128JV datasheet (Section 4.4), the `/HOLD` pin is active low. When `/HOLD` is low, the device ignores the clock, allowing the host to temporarily pause communication.
    根据 W25Q128JV 数据手册（第 4.4 节），`/HOLD` 引脚是低电平有效。当 `/HOLD` 为低时，设备会忽略时钟，允许主机暂时暂停通信。
  * If these pins are driven by other signals (e.g., shared SPI bus lines) or left floating, it can lead to unexpected behavior or communication failures.
    如果这些引脚由其他信号驱动（例如，共享的 SPI 总线线）或悬空，可能导致意外行为或通信失败。
* Connect SPI pins (SCK, MISO, MOSI) and CS pin correctly.
  正确连接 SPI 引脚 (SCK, MISO, MOSI) 和 CS 引脚。

## Usage / 使用

Add this to your `Cargo.toml`:
将以下内容添加到你的 `Cargo.toml`:

```toml
[dependencies]
w25q128 = "0.1.1" 
```