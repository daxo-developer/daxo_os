use x86_64::instructions::port::Port;
use spin::Mutex;
use crate::println;

const ATA_DATA_PORT: u16 = 0x1F0;
const ATA_SECTOR_COUNT_PORT: u16 = 0x1F2;
const ATA_LBA_LOW_PORT: u16 = 0x1F3;
const ATA_LBA_MID_PORT: u16 = 0x1F4;
const ATA_LBA_HIGH_PORT: u16 = 0x1F5;
const ATA_DRIVE_HEAD_PORT: u16 = 0x1F6;
const ATA_COMMAND_STATUS_PORT: u16 = 0x1F7;

const ATA_CMD_READ_PIO: u8 = 0x20;

const ATA_STATUS_BSY: u8 = 0x80;
const ATA_STATUS_DRQ: u8 = 0x08;
const ATA_STATUS_ERR: u8 = 0x01;

pub static ATA_BUS: Mutex<AtaBus> = Mutex::new(AtaBus::new());

pub struct AtaBus {
    data_port: Port<u16>,
    sector_count: Port<u8>,
    lba_low: Port<u8>,
    lba_mid: Port<u8>,
    lba_high: Port<u8>,
    drive_head: Port<u8>,
    command_status: Port<u8>,
}

impl AtaBus {
    const fn new() -> Self {
        Self {
            data_port: Port::new(ATA_DATA_PORT),
            sector_count: Port::new(ATA_SECTOR_COUNT_PORT),
            lba_low: Port::new(ATA_LBA_LOW_PORT),
            lba_mid: Port::new(ATA_LBA_MID_PORT),   
            lba_high: Port::new(ATA_LBA_HIGH_PORT), 
            drive_head: Port::new(ATA_DRIVE_HEAD_PORT),
            command_status: Port::new(ATA_COMMAND_STATUS_PORT),
        }
    }

    fn io_delay(&mut self) {
        unsafe {
            let mut delay_port: Port<u8> = Port::new(0x80);
            delay_port.write(0);
        }
    }

    fn wait_ready(&mut self) -> bool {
        unsafe {
            // The wait limit has been increased to allow the slow disk to respond quickly:
            for _ in 0..50000 {
                if (self.command_status.read() & ATA_STATUS_BSY) == 0 {
                    return true;
                }
                core::hint::spin_loop();
            }
        }
        false
    }

    fn wait_data(&mut self) -> bool {
        unsafe {
            for _ in 0..50000 {
                let status = self.command_status.read();
                if (status & ATA_STATUS_ERR) != 0 {
                    return false;
                }
                if (status & ATA_STATUS_DRQ) != 0 {
                    return true;
                }
                core::hint::spin_loop();
            }
        }
        false
    }

    pub fn read_sector(&mut self, lba: u32, buffer: &mut [u8; 512]) {
        unsafe {
            if self.command_status.read() == 0xFF {
                println!("[ATA] No controller detected.");
                return;
            }

            if !self.wait_ready() {
                println!("[ATA] Controller busy timeout.");
                return;
            }

            self.drive_head.write((0xE0 | ((lba >> 24) & 0x0F)) as u8);
            self.io_delay();

            self.sector_count.write(1);
            self.lba_low.write((lba & 0xFF) as u8);
            self.lba_mid.write(((lba >> 8) & 0xFF) as u8);
            self.lba_high.write(((lba >> 16) & 0xFF) as u8);

            self.command_status.write(ATA_CMD_READ_PIO);
            self.io_delay();

            if !self.wait_data() {
                println!("[ATA] Data not ready or read error.");
                return;
            }

            // Reading port data
            for i in (0..512).step_by(2) {
                let word = self.data_port.read();
                buffer[i] = (word & 0xFF) as u8;
                buffer[i + 1] = ((word >> 8) & 0xFF) as u8;
            }
        }
    }
}
