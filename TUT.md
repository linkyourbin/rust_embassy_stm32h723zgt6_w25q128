## 实现现象，展示部分调试信息
```bash
--- Starting Test Cycle / 开始测试周期 --- (w25q128 w25q128/examples/basic_usage.rs:79)
221.056457 [INFO ] JEDEC ID: Manufacturer=0xEF, Type=0x40, Capacity=0x18 / JEDEC ID: 制造商=0xEF, 类型=0x40, 容量=0x18 (w25q128 w25q128/examples/basic_usage.rs:84)
221.056518 [INFO ] ✅ Device identified successfully: W25Q128JV / ✅ 设备识别成功：W25Q128JV (w25q128 w25q128/examples/basic_usage.rs:87)
221.056579 [INFO ] Status Register 1: 0x00 / 状态寄存器1: 0x00 (w25q128 w25q128/examples/basic_usage.rs:100)
221.056579 [INFO ]   BUSY: Idle / BUSY: 空闲 (w25q128 w25q128/examples/basic_usage.rs:101)
221.056640 [INFO ]   WEL: Disabled / WEL: 未使能 (w25q128 w25q128/examples/basic_usage.rs:102)
221.056701 [INFO ]   BP[2:0]: 0x0 / BP[2:0]: 0x0 (w25q128 w25q128/examples/basic_usage.rs:103)
221.057037 [INFO ] Read 16 bytes from address 0x000000: [FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF] / 从地址0x000000读取16字节: [FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF] (w25q128 w25q128/examples/basic_usage.rs:114)
221.057128 [INFO ] ✅ Data matches expectation (unprogrammed state) / ✅ 数据符合预期（未编程状态） (w25q128 w25q128/examples/basic_usage.rs:116)
221.057464 [INFO ] Fast read 16 bytes from address 0x000000: [FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF] / 快速读取地址0x000000的16字节: [FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF] (w25q128 w25q128/examples/basic_usage.rs:130)
221.057556 [INFO ] ✅ Fast read data matches expectation (unprogrammed state) / ✅ 快速读取数据符合预期（未编程状态） (w25q128 w25q128/examples/basic_usage.rs:132)
221.057952 [INFO ] Data written successfully / 写入数据成功 (w25q128 w25q128/examples/basic_usage.rs:146)
221.058166 [INFO ] Read data after writing: [AB, CD, EF, 12, 34, 56, 78, 9A] / 写入后读取数据: [AB, CD, EF, 12, 34, 56, 78, 9A] (w25q128 w25q128/examples/basic_usage.rs:157)
221.058227 [INFO ] ✅ Written data is correct / ✅ 写入数据正确 (w25q128 w25q128/examples/basic_usage.rs:159)
221.315643 [INFO ] Sector erase successful (Address 0x000000, Size 4096 bytes) / 扇区擦除成功 (地址 0x000000, 大小 4096 bytes) (w25q128 w25q128/examples/basic_usage.rs:172)
221.315979 [INFO ] Read data after erase: [FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF] / 擦除后读取数据: [FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF, FF] (w25q128 w25q128/examples/basic_usage.rs:183)
221.316070 [INFO ] ✅ Erase successful, data restored to 0xFF / ✅ 擦除成功，数据恢复为0xFF (w25q128 w25q128/examples/basic_usage.rs:185)
226.316101 [INFO ] 
```
---
## 硬件
1. 一块`STM32`开发板，最好是板载一个`W25Qxx`模块
   1. 演示用到的开发板是UP自己设计并且焊接的一块`STM32H723ZGT6`开发板，其中`W25Q128`模块的原理图如下，可供参考
![](https://github.com/linkyourbin/rust_embassy_stm32h723zgt6_w25q128/raw/master/images/sch_w25q128.jpg)
2. 一个烧录器，推荐使用`ST-Link`
3. 若是没有板载的`W25Qxx`模块，需要自己购买一个
4. 若干杜邦线，针对独立的`W25Qxx`模块
---
## 如何使用
```bash
# 在工程目录下，打开终端执行此命令，即可添加依赖
cargo add w25q128
```
---
## 具体实践
> 在这里给出一个我的`main.rs`，你可以在[我的github仓库找到](https://github.com/linkyourbin/rust_embassy_stm32h723zgt6_w25q128)

```rust
// examples/basic_usage.rs

#![no_std]
#![no_main]

// --- Force link defmt-rtt and panic-probe ---
// --- 强制链接 defmt-rtt 和 panic-probe ---
use defmt_rtt as _; // Global import to prevent optimization / 全局导入，确保符号不被优化掉
use panic_probe as _; // Global import to prevent optimization / 全局导入，确保符号不被优化掉
// -------------------------------------------------

use defmt::{info, error, warn};
use embassy_executor::Spawner;
use embassy_stm32::{
    time::Hertz,
    gpio::{Output, Level, Speed},
    spi::{Config as SpiConfig, Spi},
};
use embassy_time::{Timer, Duration};

// Import your library / 导入你的库
use w25q128::{W25q128jv, JEDEC_MAN_ID, JEDEC_MEM_TYPE, JEDEC_CAPACITY, SECTOR_SIZE};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut peripheral_config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        peripheral_config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,
        });
        peripheral_config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV5,
            mul: PllMul::MUL160,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV2),
            divr: Some(PllDiv::DIV2),
        });
        peripheral_config.rcc.sys = Sysclk::PLL1_P;
        peripheral_config.rcc.ahb_pre = AHBPrescaler::DIV2;
        peripheral_config.rcc.apb1_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb2_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb3_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb4_pre = APBPrescaler::DIV2;
    }
    let p = embassy_stm32::init(peripheral_config);

    info!("W25Q128JV Driver Test Started / W25Q128JV驱动测试启动");

    // SPI Configuration / SPI配置
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000); // Start with 1MHz for stability / 初始用1MHz（确保稳定）
    spi_config.mode = embassy_stm32::spi::Mode {
        polarity: embassy_stm32::spi::Polarity::IdleLow,    // SPI Mode 0: Idle low / 模式0：空闲低电平
        phase: embassy_stm32::spi::Phase::CaptureOnFirstTransition, // SPI Mode 0: Sample on first edge / 模式0：第一个跳变沿采样
    };

    // Initialize SPI (Adjust pins for your hardware) / 初始化SPI（根据你的硬件调整引脚）
    let spi = Spi::new(
        p.SPI5,
        p.PF7,  // SCK
        p.PF9,  // MOSI
        p.PF8,  // MISO
        p.DMA2_CH7, // DMA TX channel (MOSI) / DMA 发送通道 (MOSI)
        p.DMA2_CH2, // DMA RX channel (MISO) / DMA 接收通道 (MISO)
        spi_config
    );

    // Initialize CS pin (Adjust pin for your hardware) / 初始化CS引脚（根据你的硬件调整引脚）
    let cs = Output::new(p.PF6, Level::High, Speed::High); // PF6 connected to /CS / PF6 连接 /CS

    // Create driver instance and initialize / 创建设备实例并初始化
    let mut flash = W25q128jv::new(spi, cs);
    flash.init().await; // Crucial: Activate CS pin / 关键：激活CS引脚

    loop {
        info!("\n--- Starting Test Cycle / 开始测试周期 ---");

        // 1. Read JEDEC ID (Verify communication) / 读取JEDEC ID（验证通信正确性）
        match flash.read_jedec_id().await {
            Ok((man_id, mem_type, capacity)) => {
                info!("JEDEC ID: Manufacturer=0x{:02X}, Type=0x{:02X}, Capacity=0x{:02X} / JEDEC ID: 制造商=0x{:02X}, 类型=0x{:02X}, 容量=0x{:02X}",
                    man_id, mem_type, capacity, man_id, mem_type, capacity);
                if man_id == JEDEC_MAN_ID && mem_type == JEDEC_MEM_TYPE && capacity == JEDEC_CAPACITY {
                    info!("✅ Device identified successfully: W25Q128JV / ✅ 设备识别成功：W25Q128JV");
                } else {
                    warn!("❌ ID mismatch, possible communication error / ❌ ID不匹配，可能通信异常");
                }
            }
            Err(e) => {
                error!("Failed to read JEDEC ID: {:?} / 读取JEDEC ID失败: {:?}", e, e);
            }
        }

        // 2. Read Status Register (Verify device status) / 读取状态寄存器（验证设备状态）
        match flash.read_status_register().await {
            Ok(status) => {
                info!("Status Register 1: 0x{:02X} / 状态寄存器1: 0x{:02X}", status, status);
                info!("  BUSY: {} / BUSY: {}", if (status & 0x01) != 0 { "Busy" } else { "Idle" }, if (status & 0x01) != 0 { "忙" } else { "空闲" });
                info!("  WEL: {} / WEL: {}", if (status & 0x02) != 0 { "Enabled" } else { "Disabled" }, if (status & 0x02) != 0 { "已使能" } else { "未使能" });
                info!("  BP[2:0]: 0x{:01X} / BP[2:0]: 0x{:01X}", (status >> 2) & 0x07, (status >> 2) & 0x07); // Block Protect Bits / 块保护位
            }
            Err(e) => {
                error!("Failed to read Status Register: {:?} / 读取状态寄存器失败: {:?}", e, e);
            }
        }

        // 3. Read data (Should be 0xFF if unprogrammed) / 读取数据（默认应为0xFF，未擦除状态）
        let mut read_buf = [0u8; 16];
        match flash.read_data(0x000000, &mut read_buf).await {
            Ok(()) => {
                info!("Read 16 bytes from address 0x000000: {:02X} / 从地址0x000000读取16字节: {:02X}", read_buf, read_buf);
                if read_buf.iter().all(|&x| x == 0xFF) {
                    info!("✅ Data matches expectation (unprogrammed state) / ✅ 数据符合预期（未编程状态）");
                } else {
                    info!("📌 Data is not all 0xFF (may be programmed or partially erased) / 📌 数据非全FF（可能已编程或部分擦除）");
                }
            }
            Err(e) => {
                error!("Failed to read data: {:?} / 读取数据失败: {:?}", e, e);
            }
        }

        // 4. Fast Read data (Test fast read functionality) / 快速读取数据（测试快速读取功能）
        let mut fast_read_buf = [0u8; 16];
        match flash.fast_read(0x000000, &mut fast_read_buf).await {
            Ok(()) => {
                info!("Fast read 16 bytes from address 0x000000: {:02X} / 快速读取地址0x000000的16字节: {:02X}", fast_read_buf, fast_read_buf);
                if fast_read_buf.iter().all(|&x| x == 0xFF) {
                    info!("✅ Fast read data matches expectation (unprogrammed state) / ✅ 快速读取数据符合预期（未编程状态）");
                } else {
                    info!("📌 Fast read data is not all 0xFF / 📌 快速读取数据非全FF");
                }
            }
            Err(e) => {
                error!("Failed to fast read data: {:?} / 快速读取数据失败: {:?}", e, e);
            }
        }

        // 5. Write data / 写入数据
        let write_data = [0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x9A];
        match flash.write_data(0x000000, &write_data).await {
            Ok(()) => {
                info!("Data written successfully / 写入数据成功");
            }
            Err(e) => {
                error!("Failed to write data: {:?} / 写入数据失败: {:?}", e, e);
            }
        }

        // 6. Read data after writing / 读取写入的数据
        let mut read_after_write = [0u8; 8];
        match flash.read_data(0x000000, &mut read_after_write).await {
            Ok(()) => {
                info!("Read data after writing: {:02X} / 写入后读取数据: {:02X}", read_after_write, read_after_write);
                if read_after_write == write_data {
                    info!("✅ Written data is correct / ✅ 写入数据正确");
                } else {
                    warn!("❌ Written data mismatch / ❌ 写入数据不匹配");
                }
            }
            Err(e) => {
                error!("Failed to read written data: {:?} / 读取写入数据失败: {:?}", e, e);
            }
        }

        // 7. Sector Erase (Note address alignment) / 扇区擦除 (注意地址对齐)
        match flash.erase_sector(0x000000).await { // 0x000000 is 4KB sector-aligned / 0x000000 是 4KB 扇区对齐的
            Ok(()) => {
                info!("Sector erase successful (Address 0x000000, Size {} bytes) / 扇区擦除成功 (地址 0x000000, 大小 {} bytes)", SECTOR_SIZE, SECTOR_SIZE);
            }
            Err(e) => {
                error!("Sector erase failed: {:?} / 扇区擦除失败: {:?}", e, e);
            }
        }

        // 8. Verify erase result / 验证擦除结果
        let mut read_after_erase = [0u8; 16];
        match flash.read_data(0x000000, &mut read_after_erase).await {
            Ok(()) => {
                info!("Read data after erase: {:02X} / 擦除后读取数据: {:02X}", read_after_erase, read_after_erase);
                if read_after_erase.iter().all(|&x| x == 0xFF) {
                    info!("✅ Erase successful, data restored to 0xFF / ✅ 擦除成功，数据恢复为0xFF");
                } else {
                    warn!("❌ Erase failed, data not restored to 0xFF / ❌ 擦除失败，数据未恢复为0xFF");
                }
            }
            Err(e) => {
                error!("Failed to read data after erase: {:?} / 读取擦除后数据失败: {:?}", e, e);
            }
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}
```
---
## 注意事项
- 以上代码，几乎是求助`LLM`完成的
  - 只是完成了部分功能，也只在我的这块特定的开发板，还有这个特定的`W25Q128`模块上进行了测试
    - 若是需要应用到实际生产中，还需要进一步评估源码实现的合理性
  - 如果你对这个模块有更好的实现方式，还望不吝赐教
  - 也欢迎大家进行测试和完善