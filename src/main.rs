#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::cell::RefCell;
use core::fmt::Write;
use psx::{gpu::VideoMode, include_words, Framebuffer};

// SPU Control and Status Register
const SPU_CONTROL: u32 = 0x1F801DAA;
const SPU_STATUS: u32 = 0x1F801DAE;

// SPU Memory Access
const SPU_RAM_DATA_TRANSFER_ADDRESS: u32 = 0x1F801DA6;
const SPU_RAM_DATA_TRANSFER_FIFO: u32 = 0x1F801DA8;
const SPU_RAM_DATA_TRANSFER_CONTROL: u32 = 0x1F801DAC;

// SPU Interrupt
const SPU_IRQ_ADDRESS: u32 = 0x1F801DA4;

// SPU Voice Registers
const SPU_VOICE_0_ADPCM_START_ADDRESS: u32 = 0x1F801C06;
const SPU_VOICE_0_ADPCM_REPEAT_ADDRESS: u32 = 0x1F801C0E;
const SPU_VOICE_0_ADPCM_SAMPLE_RATE: u32 = 0x1F801C04;
const SPU_VOICE_0_ADSR: u32 = 0x1F801C08;
const SPU_VOICE_0_VOLUME_LEFT: u32 = 0x1F801C00;
const SPU_VOICE_0_VOLUME_RIGHT: u32 = 0x1F801C02;
const SPU_VOICE_0_CURRENT_ADSR_VOLUME: u32 = 0x1F801C0C;

// SPU Voice Flags
const SPU_VOICE_KEY_ON: u32 = 0x1F801D88;
const SPU_VOICE_KEY_OFF: u32 = 0x1F801D8C;
const SPU_VOICE_ON_OFF_STATUS: u32 = 0x1F801D9C;

// SPU Noise Generator
const SPU_NOISE_MODE_ENABLE: u32 = 0x1F801D94;

// SPU Volume and ADSR Generator
const SPU_MAIN_VOLUME_LEFT: u32 = 0x1F801D80;
const SPU_MAIN_VOLUME_RIGHT: u32 = 0x1F801D82;
const SPU_CD_AUDIO_INPUT_VOLUME: u32 = 0x1F801DB0;
const SPU_EXTERNAL_AUDIO_INPUT_VOLUME: u32 = 0x1F801DB4;

// SPU Reverb Registers
const SPU_REVERB_OUTPUT_VOLUME_LEFT: u32 = 0x1F801D84;
const SPU_REVERB_OUTPUT_VOLUME_RIGHT: u32 = 0x1F801D86;
const SPU_REVERB_WORK_AREA_START_ADDRESS: u32 = 0x1F801DA2;
const SPU_REVERB_APF_OFFSET_1: u32 = 0x1F801DC0;
const SPU_REVERB_APF_OFFSET_2: u32 = 0x1F801DC2;
const SPU_REVERB_REFLECTION_VOLUME_1: u32 = 0x1F801DC4;
const SPU_REVERB_COMB_VOLUME_1: u32 = 0x1F801DC6;
const SPU_REVERB_COMB_VOLUME_2: u32 = 0x1F801DC8;
const SPU_REVERB_COMB_VOLUME_3: u32 = 0x1F801DCA;
const SPU_REVERB_COMB_VOLUME_4: u32 = 0x1F801DCC;
const SPU_REVERB_REFLECTION_VOLUME_2: u32 = 0x1F801DCE;
const SPU_REVERB_APF_VOLUME_1: u32 = 0x1F801DD0;
const SPU_REVERB_APF_VOLUME_2: u32 = 0x1F801DD2;
const SPU_REVERB_SAME_SIDE_REFLECTION_ADDRESS_1_LEFT: u32 = 0x1F801DD4;
const SPU_REVERB_SAME_SIDE_REFLECTION_ADDRESS_1_RIGHT: u32 = 0x1F801DD6;
const SPU_REVERB_COMB_ADDRESS_1_LEFT: u32 = 0x1F801DD8;
const SPU_REVERB_COMB_ADDRESS_1_RIGHT: u32 = 0x1F801DDA;
const SPU_REVERB_COMB_ADDRESS_2_LEFT: u32 = 0x1F801DDC;
const SPU_REVERB_COMB_ADDRESS_2_RIGHT: u32 = 0x1F801DDE;
const SPU_REVERB_SAME_SIDE_REFLECTION_ADDRESS_2_LEFT: u32 = 0x1F801DE0;
const SPU_REVERB_SAME_SIDE_REFLECTION_ADDRESS_2_RIGHT: u32 = 0x1F801DE2;
const SPU_REVERB_DIFFERENT_SIDE_REFLECTION_ADDRESS_1_LEFT: u32 = 0x1F801DE4;
const SPU_REVERB_DIFFERENT_SIDE_REFLECTION_ADDRESS_1_RIGHT: u32 = 0x1F801DE6;
const SPU_REVERB_COMB_ADDRESS_3_LEFT: u32 = 0x1F801DE8;
const SPU_REVERB_COMB_ADDRESS_3_RIGHT: u32 = 0x1F801DEA;
const SPU_REVERB_COMB_ADDRESS_4_LEFT: u32 = 0x1F801DEC;
const SPU_REVERB_COMB_ADDRESS_4_RIGHT: u32 = 0x1F801DEE;
const SPU_REVERB_DIFFERENT_SIDE_REFLECTION_ADDRESS_2_LEFT: u32 = 0x1F801DF0;
const SPU_REVERB_DIFFERENT_SIDE_REFLECTION_ADDRESS_2_RIGHT: u32 = 0x1F801DF2;
const SPU_REVERB_APF_ADDRESS_1_LEFT: u32 = 0x1F801DF4;
const SPU_REVERB_APF_ADDRESS_1_RIGHT: u32 = 0x1F801DF6;
const SPU_REVERB_APF_ADDRESS_2_LEFT: u32 = 0x1F801DF8;
const SPU_REVERB_APF_ADDRESS_2_RIGHT: u32 = 0x1F801DFA;
const SPU_REVERB_INPUT_VOLUME_LEFT: u32 = 0x1F801DFC;
const SPU_REVERB_INPUT_VOLUME_RIGHT: u32 = 0x1F801DFE;

// SPU Unknown Registers
const SPU_UNKNOWN_STATUS_REGISTER: u32 = 0x1F801DA0;
const SPU_UNKNOWN_REGISTER_1: u32 = 0x1F801DBC;
const SPU_UNKNOWN_REGISTER_2: u32 = 0x1F801E60;

pub struct MemoryCell<T>(*mut T);
impl<T> MemoryCell<T> {
    pub const fn new(address: usize) -> Self {
        MemoryCell(address as *mut T)
    }
    
    pub fn get(&self) -> T {
        unsafe { self.0.read_volatile() }
    }
    
    pub fn set(&self, x: T) {
        unsafe { self.0.write_volatile(x) }
    }
}


struct SpuReg;
impl SpuReg {
    fn write<T>(addr: u32, value: T) {
        MemoryCell::<T>::new(addr as usize).set(value);
    }
    
    fn read<T>(addr: u32) -> T {
        MemoryCell::<T>::new(addr as usize).get()
    }
}

#[unsafe(no_mangle)]
fn main() {
    play_adpcm();

    loop {
        unsafe { core::arch::asm!("nop") }
    }
}

pub fn play_adpcm() {
    // Valid ADPCM block with loop flags (Bit0=End, Bit1=Repeat)
    let audio_data: &[u32] = include_words!("./../assets/audio/test.adpcm");

    let spu_ram_base = 0x1000; // 8-byte aligned (0x1000/8 = 0x200)

    // 1. SPU Initialization - Critical unmute
    SpuReg::write(SPU_CONTROL, 3u32 << 14); // SPU Enable (bit15=1), Unmute (bit14=0)
    wait(1000); // Extended initialization delay

    // 2. Configure transfer control
    SpuReg::write(SPU_RAM_DATA_TRANSFER_CONTROL, 0x0000u16); // Stop mode first
    SpuReg::write(SPU_RAM_DATA_TRANSFER_ADDRESS, (spu_ram_base / 8) as u16);
    SpuReg::write(SPU_RAM_DATA_TRANSFER_CONTROL, 0x0001u16); // Manual write mode

    // 3. Transfer data with proper FIFO handling
    for &word in audio_data.iter() {
        let parts = [(word & 0xFFFF) as u16, (word >> 16) as u16];
        for part in parts {
            // Wait until FIFO is ready (bit10=0)
            while SpuReg::read::<u16>(SPU_STATUS) & 0x400 != 0 {}
            SpuReg::write(SPU_RAM_DATA_TRANSFER_FIFO, part);
        }
    }
    SpuReg::write(SPU_RAM_DATA_TRANSFER_CONTROL, 0x0000u16); // Return to stop mode

    // 4. Configure voice parameters
    let spu_addr = (spu_ram_base / 8) as u16;
    SpuReg::write(SPU_VOICE_0_ADPCM_START_ADDRESS, spu_addr);
    SpuReg::write(SPU_VOICE_0_ADPCM_REPEAT_ADDRESS, spu_addr);
    
    // ADSR: Attack=Fast, Decay=Medium, Sustain=Max, Release=Slow
    // Lower 16-bit: Attack=Exponential, Shift=4, Step=7 | Decay Shift=4 | Sustain Level=0xF
    // Upper 16-bit: Sustain=Exponential Increase, Shift=4, Step=7 | Release Shift=4
    SpuReg::write(SPU_VOICE_0_ADSR, 0x80E8_1FE0u32); // Corrected ADSR envelope
    
    // Set sample rate to 44100Hz (0x1000)
    SpuReg::write(SPU_VOICE_0_ADPCM_SAMPLE_RATE, 0x1000u16);

    // 5. Set volumes
    SpuReg::write(SPU_MAIN_VOLUME_LEFT, 0x3FFFu16);
    SpuReg::write(SPU_MAIN_VOLUME_RIGHT, 0x3FFFu16);
    SpuReg::write(SPU_VOICE_0_VOLUME_LEFT, 0x3FFFu16); // Fixed volume mode
    SpuReg::write(SPU_VOICE_0_VOLUME_RIGHT, 0x3FFFu16);

    // 6. Voice activation sequence
    SpuReg::write(SPU_VOICE_KEY_OFF, 0xFFFFu16); // Reset all voices
    wait(500);
    SpuReg::write(SPU_VOICE_KEY_ON, 0x0001u16); // Activate voice 0
    wait(10000); // Allow time for playback start
}

fn wait(cycles: u32) {
    for _ in 0..cycles {
        unsafe { core::arch::asm!("nop") }
    }
}

fn init_debug() -> impl Fn(&str) {
    let buf0 = (0, 0);
    let buf1 = (0, 240);
    let res = (320, 240);
    let txt_offset = (0, 64);
    let mut fb = Framebuffer::new(buf0, buf1, res, VideoMode::NTSC, None).unwrap();
    let font = fb.load_default_font();
    let txt = RefCell::new((font.new_text_box(txt_offset, res), fb));

    move |message: &str| {
        let mut txt = txt.borrow_mut();
        txt.0.reset();
        for ch in message.as_bytes() {
            txt.0.print_char(ch.clone());
            txt.0.move_right(1);
        }
        
        txt.1.draw_sync();
        txt.1.wait_vblank();
        txt.1.swap();
    }
}