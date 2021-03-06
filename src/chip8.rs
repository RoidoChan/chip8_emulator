// Copyright 2020 Lloyd Crawley

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and 
// associated documentation files (the "Software"), to deal in the Software without restriction, 
// including without limitation the rights to use, copy, modify, merge, publish, distribute, 
// sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is 
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or 
// substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, 
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR 
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE 
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, 
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE 
// SOFTWARE.

//chip8.rs
use crate::cpu;

// memory map
const MEM_BEGIN : u16 = 0x200;
const DISPLAY_REFRESH : u16 = 0xF00;
const MISC : u16 = 0xEA0;

const FREQ : u32 = 16_000_000;

const chip8_fontset : [u8; 80] =
[
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];


#[derive(Debug, Default)]
struct Timer{
    count : u8,
    current_time : u32,
    timer_complete : bool
}

impl Timer{
    pub fn update(&mut self, deltaTime: u32){
        self.current_time += deltaTime;

        if self.current_time > FREQ{
            self.current_time -= FREQ;
            if  self.count > 0{
                self.count -= 1;
                if self.count == 0 {
                    println!("timer triggered!");
                    self.timer_complete = true;
                }
            }else{
                println!("timer switched off!");
                self.timer_complete = false;
            }
        }
    }

    pub fn complete(&self) -> bool {
        println!("check complete {}", self.timer_complete);
        self.timer_complete
    }
}

#[derive(Debug, Default)]
struct Memory{
    ram  : Box<[u8]>,
    vram : Box<[u8]>
}

impl Memory{
    pub fn default() -> Self{
        Memory{
            ram : vec![0; 4096].into_boxed_slice(),
            vram : vec![0; 8 * 32].into_boxed_slice() // 32 * 64 BITS hence the small num of bytes
        }
    }
}

#[derive(Debug, Default)]
pub struct hw_bundle{
    delayTimer : Timer,
    soundTimer : Timer,
    memory     : Memory,
    keys       : [u8; 16],
}

impl hw_bundle {

    pub fn default() -> Self{
        hw_bundle{
            delayTimer : Timer::default(),
            soundTimer : Timer::default(),
            memory     : Memory::default(),
            keys       : [0; 16],
        }
    }

     // 1 for down, 0 for up
    pub fn set_key(&mut self, key_code : u8, up: u8){
        self.keys[key_code as usize] = up; 
    }

    pub fn play_sound(&mut self) -> bool {
        self.soundTimer.complete()
    }

    pub fn get_vram(&self) -> &Box<[u8]>{
        &self.memory.vram
    }

    pub fn read_ram_value(&self, address: usize) -> u8{
        self.memory.ram[address]
    }

    pub fn write_ram_value(&mut self, address: usize, value : u8){
        self.memory.ram[address] = value;
    }

    pub fn xor_vram_value(&mut self, address: usize, value: u8){
        self.memory.vram[address] ^= value;
    }

    pub fn read_vram_value(&self, address: usize) -> u8{
            self.memory.vram[address]
    }

    pub fn write_vram_value(&mut self, address: usize, value : u8){
        self.memory.vram[address] = value;
    }

    pub fn get_delay_timer_count(&self) -> u8 {
        self.delayTimer.count
    }

    pub fn get_sound_timer_count(&self) -> u8 {
        self.soundTimer.count
    }

    pub fn set_delay_timer_count(&mut self, value : u8) {
        self.delayTimer.count = value;
    }

    pub fn set_sound_timer_count(&mut self, value : u8) {
        println!("set sound timer to {}", value);
        self.soundTimer.count = value;
    }

    pub fn read_key(&self, key : usize) -> u8 {
        self.keys[key]
    }

    pub fn load_rom(&mut self, rom : &Box<[u8]>){
        let mut mem_start : usize = 0x200;
        for i in 0..rom.len() {
            self.memory.ram[mem_start] = rom[i as usize];
            mem_start += 1;
        }

        // also load font
        for i in 0..chip8_fontset.len() {
            self.memory.ram[i] = chip8_fontset[i];
        }
    }

    pub fn run(&mut self, deltaTime: u32){
        self.delayTimer.update(deltaTime);
        self.soundTimer.update(deltaTime);
    }
}

#[derive(Debug)]
pub struct Chip_HW{
    cpu        : cpu::CPU,
    pub hw         : hw_bundle
}

impl Chip_HW{
    pub fn new( ) -> Self{
        Chip_HW{
            cpu     : cpu::CPU::new(),
            hw      : hw_bundle::default()
        }
    }

    pub fn run(&mut self, deltaTime: u32){
        self.cpu.next_instruction(&mut self.hw);
        self.hw.run(deltaTime);
    }
}