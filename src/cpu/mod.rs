pub mod regs;
use core::panic;

#[cfg(feature = "log")]
use log::{debug, error, trace};
use regs::Registers;

#[cfg(feature = "symlog")]
use symlog::log;
#[macro_export]
macro_rules! getLsb {
    ($a:expr) => {
        !($a - 1) & $a
    };
}
pub struct CPU {
    pub instruction: u8,
    regs: Registers,
    input: [u8; 0x100],
    pub out_strobe:(bool, u8, u8),
    pub interrupt_enabled:bool,
}
impl CPU {
    pub fn new(start_pc: u16, start_sp: u16) -> CPU {
        let mut cpu = CPU {
            instruction: 0,
            regs: Registers::default(),
            input: [0x00; 0x100],
            out_strobe:(false, 0, 0),
            interrupt_enabled:false
        };
        cpu.regs.pc = start_pc;
        cpu.regs.sp = start_sp;
        cpu
    }
    pub fn set_input_n(&mut self, n: u8, value: u8) {
        self.input[n as usize] = value;
    }
    pub fn get_regs(&self) -> Registers {
        self.regs
    }
    fn get_16(&self, mem: &[u8]) -> u16 {
        let lb = mem[self.regs.pc as usize + 1];
        let hb = mem[self.regs.pc as usize + 2];
        (hb as u16) << 8 | lb as u16
    }
    fn pop_16(&mut self, mem: &[u8]) -> u16 {
        let lb = mem[self.regs.sp as usize];
        let hb = mem[self.regs.sp as usize + 1];
        self.regs.sp += 2;
        (hb as u16) << 8 | lb as u16
    }
    pub fn next(&mut self, mem: &mut [u8]) {
        #[cfg(feature = "symlog")]
        log!(PC);
        #[cfg(feature = "log")]
        debug!("PC: {:04X} ", self.regs.pc);
        #[cfg(feature = "std")]
        print!("PC: {:04X} ", self.regs.pc);
        self.instruction = mem[self.regs.pc as usize];
        LUT[self.instruction as usize](self, mem);
        #[cfg(feature = "symlog")]
        log!(REGS);
        #[cfg(feature = "log")]
        trace!("{:X?}, {:X?}\n",self.instruction, self.regs);
        #[cfg(feature = "std")]
        print!("{:X?}\n", self.regs);
    }
    fn jmp(&mut self, mem: &mut [u8]) {
        let addr = self.get_16(mem);
        self.regs.pc = addr;
        #[cfg(feature = "symlog")]
        log!(JMP);
        #[cfg(feature = "log")]
        debug!("JMP {:04X}", addr);
    }
    fn lxi(&mut self, mem: &mut [u8]) {
        let val = self.get_16(mem);
        self.regs.set_rp(val, self.instruction);
        self.regs.pc += 3;
        #[cfg(feature = "symlog")]
        log!(LXI);
        #[cfg(feature = "log")]
        debug!("LXI {:04X}", val);
    }
    fn ani(&mut self, mem: &mut [u8]) {
        let db = mem[self.regs.pc as usize + 1];
        self.regs.a &= db;
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(ANI);
        #[cfg(feature = "log")]
        debug!("ANI {:02X}", db);
    }
    fn jccc(&mut self, mem: &mut [u8]) {
        let mut addr = 0;
        if self.regs.cond(self.instruction) {
            addr = self.get_16(mem);
            self.regs.pc = addr;
        } else {
            self.regs.pc += 3;
        }
        #[cfg(feature = "symlog")]
        log!(Jccc);
        #[cfg(feature = "log")]
        debug!("Jccc {:04X}", addr);
    }
    fn adi(&mut self, mem: &mut [u8]) {
        let db = mem[self.regs.pc as usize + 1];
        let (a, v) = self.regs.a.overflowing_add(db);
        let h = ((self.regs.a & 0xF) + (db & 0xF)) & 0x10 == 0x10;
        self.regs.a = a;
        self.regs.set_flags(self.regs.a, v, h);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(ADI);
        #[cfg(feature = "log")]
        debug!("ADI {:02X}", db);
    }
    fn call(&mut self, mem: &mut [u8]) {
        mem[self.regs.sp as usize - 1] = (self.regs.pc >> 8) as u8;
        mem[self.regs.sp as usize - 2] = self.regs.pc as u8;
        self.regs.sp -= 2;
        let addr = self.get_16(mem);
        self.regs.pc = addr;
        #[cfg(feature = "symlog")]
        log!(CALL);
        #[cfg(feature = "log")]
        debug!("CALL {:04X}", addr);
    }
    fn push(&mut self, mem: &mut [u8]) {
        let rp = self.regs.get_rp(self.instruction);
        mem[self.regs.sp as usize - 1] = (rp >> 8) as u8;
        mem[self.regs.sp as usize - 2] = rp as u8;
        self.regs.sp -= 2;
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(PUSH);
        #[cfg(feature = "log")]
        debug!("PUSH {:04X}", rp)
    }
    fn xchg(&mut self, _mem: &mut [u8]) {
        let hl = self.regs.get_rp(0x20);
        let de = self.regs.get_rp(0x10);
        self.regs.set_rp(hl, 0x10);
        self.regs.set_rp(de, 0x20);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(XCHG);
        #[cfg(feature = "log")]
        debug!("XCHG {:04X}", de);
    }
    fn mvi(&mut self, mem: &mut [u8]) {
        self.regs.set_d(self.instruction, mem, mem[self.regs.pc as usize + 1]);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(MVI);
        #[cfg(feature = "log")]
        debug!("MVI {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn nop(&mut self, _mem: &mut [u8]) {
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(NOP);
        #[cfg(feature = "log")]
        debug!("NOP {:04X}", self.regs.pc);
    }
    fn fault(&mut self, _mem: &mut [u8]) {
        #[cfg(feature = "log")]
        error!(
            "regs:{:x?}, instr:{:08b}, {:02x}",
            self.regs, self.instruction, self.instruction
        );
        panic!("FAULT");
    }
    fn mov(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        self.regs.set_d(self.instruction, mem, s);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(MOV);
        #[cfg(feature = "log")]
        debug!("MOV {:02X}", s);
    }
    fn lda(&mut self, mem: &mut [u8]) {
        let addr = self.get_16(mem);
        self.regs.a = mem[addr as usize];
        self.regs.pc += 3;
        #[cfg(feature = "symlog")]
        log!(LDA);
        #[cfg(feature = "log")]
        debug!("LDA {:04X}", addr);
    }
    fn sda(&mut self, mem: &mut [u8]) {
        let addr = self.get_16(mem);
        mem[addr as usize] = self.regs.a;
        self.regs.pc += 3;
        #[cfg(feature = "symlog")]
        log!(SDA);
        #[cfg(feature = "log")]
        debug!("SDA {:04X}", addr);
    }
    fn lhld(&mut self, mem: &mut [u8]) {
        let addr = self.get_16(mem) as usize;
        let val = (mem[addr + 1] as u16) << 8 | mem[addr] as u16;
        self.regs.set_rp(val, 0x20);
        self.regs.pc += 3;
        #[cfg(feature = "symlog")]
        log!(LHLD);
        #[cfg(feature = "log")]
        debug!("LHLD {:04X}", val);
    }
    fn shld(&mut self, mem: &mut [u8]) {
        let addr = self.get_16(mem) as usize;
        let val = self.regs.get_rp(0x20);
        mem[addr] = val as u8;
        mem[addr + 1] = (val >> 8) as u8;
        self.regs.pc += 3;
        #[cfg(feature = "symlog")]
        log!(SHLD);
        #[cfg(feature = "log")]
        debug!("SHLD {:04X}", val);
    }
    fn ldax(&mut self, mem: &mut [u8]) {
        let rp = self.regs.get_rp(self.instruction);
        self.regs.a = mem[rp as usize];
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(LDAX);
        #[cfg(feature = "log")]
        debug!("LDAX {:04X}", rp);
    }
    fn stax(&mut self, mem: &mut [u8]) {
        let rp = self.regs.get_rp(self.instruction);
        mem[rp as usize] = self.regs.a;
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(STAX);
        #[cfg(feature = "log")]
        debug!("STAX {:04X}", rp);
    }
    fn add(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        let (a, v) = self.regs.a.overflowing_add(s);
        let h = ((self.regs.a & 0xF) + (s & 0xF)) & 0x10 == 0x10;
        self.regs.set_flags(a, v, h);
        self.regs.a = a;
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(ADD);
        #[cfg(feature = "log")]
        debug!("ADD {:02X}", s);
    }
    fn adc(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        let (a0, v0) = self.regs.a.overflowing_add(s);
        let (a1, v1) = a0.overflowing_add(self.regs.f.get_carry() as u8);
        let h = ((self.regs.a & 0xF) + (s & 0xF) + self.regs.f.get_carry() as u8) & 0x10 == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0 | v1, h);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(ADC);
        #[cfg(feature = "log")]
        debug!("ADC {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn aci(&mut self, mem: &mut [u8]) {
        let s = mem[self.regs.pc as usize + 1];
        let (a0, v0) = self.regs.a.overflowing_add(s);
        let (a1, v1) = a0.overflowing_add(self.regs.f.get_carry() as u8);
        let h = ((self.regs.a & 0xF) + (s & 0xF) + self.regs.f.get_carry() as u8) & 0x10 == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0 | v1, h);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(ACI);
        #[cfg(feature = "log")]
        debug!("ACI {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn sub(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        let (a, v) = self.regs.a.overflowing_sub(s);
        let h = (self.regs.a & 0xF).wrapping_sub(s & 0xF) & 0x10 == 0x10;
        self.regs.set_flags(a, v, h);
        self.regs.a = a;
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(SUB);
        #[cfg(feature = "log")]
        debug!("SUB {:02X}", s);
    }
    fn sui(&mut self, mem: &mut [u8]) {
        let s = mem[self.regs.pc as usize + 1];
        let (a, v) = self.regs.a.overflowing_sub(s);
        let h = ((self.regs.a & 0xF) + (s & 0xF)) & 0x10 == 0x10;
        self.regs.a = a;
        self.regs.set_flags(self.regs.a, v, h);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(SUI);
        #[cfg(feature = "log")]
        debug!("SUI {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn sbb(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        let (a0, v0) = self.regs.a.overflowing_sub(s);
        let (a1, v1) = a0.overflowing_sub(self.regs.f.get_carry() as u8);
        let h = (self.regs.a & 0xF)
            .wrapping_sub(s & 0xF)
            .wrapping_sub(self.regs.f.get_carry() as u8)
            & 0x10
            == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0 | v1, h);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(SBB);
        #[cfg(feature = "log")]
        debug!("SBB {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn sbi(&mut self, mem: &mut [u8]) {
        let s = mem[self.regs.pc as usize + 1];
        let (a0, v0) = self.regs.a.overflowing_sub(s);
        let (a1, v1) = a0.overflowing_sub(self.regs.f.get_carry() as u8);
        let h = (self.regs.a & 0xF)
            .wrapping_sub(s & 0xF)
            .wrapping_sub(self.regs.f.get_carry() as u8)
            & 0x10
            == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0 | v1, h);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(SBI);
        #[cfg(feature = "log")]
        debug!("SBI {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn inr(&mut self, mem: &mut [u8]) {
        let r = self.regs.get_d(self.instruction, mem);
        let i = r.wrapping_add(1);
        let h = ((r & 0xF) + 1) & 0x10 == 0x10;
        self.regs.set_d(self.instruction, mem, i);
        self.regs.set_flags(i, false, h);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(INR);
        #[cfg(feature = "log")]
        debug!("INR {:02X}", r);
    }
    fn dcr(&mut self, mem: &mut [u8]) {
        let r = self.regs.get_d(self.instruction, mem);
        let i = r.wrapping_sub(1);
        let h = ((r & 0xF).wrapping_sub(1)) & 0x10 == 0x10;
        self.regs.set_d(self.instruction, mem, i);
        self.regs.set_flags(i, false, h);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(DCR);
        #[cfg(feature = "log")]
        debug!("DCR {:02X}", i);
    }
    fn inx(&mut self, _mem: &mut [u8]) {
        let rp = self.regs.get_rp(self.instruction);
        self.regs.set_rp(rp.wrapping_add(1), self.instruction);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(INX);
        #[cfg(feature = "log")]
        debug!("INX {:02x}", rp);
    }
    fn dcx(&mut self, _mem: &mut [u8]) {
        let rp = self.regs.get_rp(self.instruction);
        self.regs.set_rp(rp.wrapping_sub(1), self.instruction);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(DCX);
        #[cfg(feature = "log")]
        debug!("DCX {:02x}", rp);
    }
    fn dad(&mut self, _mem: &mut [u8]) {
        let rp = self.regs.get_rp(self.instruction);
        let hl = self.regs.get_rp(0x20);
        let (hl, v) = hl.overflowing_add(rp);
        self.regs.set_rp(hl, 0x20);
        self.regs.f.set_carry(v);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(DAD);
        #[cfg(feature = "log")]
        debug!("DAD {:04x}", hl);
    }
    fn daa(&mut self, _mem: &mut [u8]) {
        #[cfg(feature = "log")]
        error!("DAA");
        //panic!("DAA at addr {:02X}", self.regs.pc);
    }
    fn ana(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        self.regs.a &= s;
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(ANA);
        #[cfg(feature = "log")]
        debug!("ANA {:02X}", s);
    }
    fn ora(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        self.regs.a |= s;
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(ORA);
        #[cfg(feature = "log")]
        debug!("ORA {:02X}", s);
    }
    fn ori(&mut self, mem: &mut [u8]) {
        self.regs.a |= mem[self.regs.pc as usize + 1];
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(ORI);
        #[cfg(feature = "log")]
        debug!("ORI {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn xra(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        self.regs.a ^= s;
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(XRA);
        #[cfg(feature = "log")]
        debug!("XRA {:02X}", s);
    }
    fn xri(&mut self, mem: &mut [u8]) {
        self.regs.a ^= mem[self.regs.pc as usize + 1];
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(XRI);
        #[cfg(feature = "log")]
        debug!("XRI {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn cmp(&mut self, mem: &mut [u8]) {
        let s = self.regs.get_s(self.instruction, mem);
        let h = (self.regs.a & 0xF).wrapping_sub(s) & 0x10 == 0x10;
        let (a, v) = self.regs.a.overflowing_sub(s);
        self.regs.set_flags(a, v, h);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(CMP);
        #[cfg(feature = "log")]
        debug!("CMP {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn cpi(&mut self, mem: &mut [u8]) {
        let s = mem[self.regs.pc as usize + 1];
        let h = (self.regs.a & 0xF).wrapping_sub(s) & 0x10 == 0x10;
        let (a, v) = self.regs.a.overflowing_sub(s);
        self.regs.set_flags(a, v, h);
        self.regs.pc += 2;
        #[cfg(feature = "symlog")]
        log!(CPI);
        #[cfg(feature = "log")]
        debug!("CPI {:02X}", mem[self.regs.pc as usize + 1]);
    }
    fn rlc(&mut self, _mem: &mut [u8]) {
        let (a, c) = self.regs.a.overflowing_mul(2);
        self.regs.a = a + c as u8;
        self.regs.f.set_carry(c);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(RLC);
        #[cfg(feature = "log")]
        debug!("RLC {:02x}", a);
    }
    fn rrc(&mut self, _mem: &mut [u8]) {
        let a = self.regs.a.rotate_right(1);
        self.regs.a = a;
        self.regs.f.set_carry(a & 0x80 == 0x80);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(RRC);
        #[cfg(feature = "log")]
        debug!("RRC {:02x}", a);
    }
    fn ral(&mut self, _mem: &mut [u8]) {
        let (a, c) = self.regs.a.overflowing_mul(2);
        self.regs.a = a + self.regs.f.get_carry() as u8;
        self.regs.f.set_carry(c);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(RAL);
        #[cfg(feature = "log")]
        debug!("RAL {:02x}", a);
    }
    fn rar(&mut self, _mem: &mut [u8]) {
        self.regs.f.set_carry(self.regs.a & 1 == 1);
        let a = (self.regs.a as i8) >> 1;
        self.regs.a = a as u8;
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(RAR);
        #[cfg(feature = "log")]
        debug!("RAL {:02x}", a);
    }
    fn cma(&mut self, _mem: &mut [u8]) {
        self.regs.a = !self.regs.a;
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(CMA);
        #[cfg(feature = "log")]
        debug!("CMA {:02x}", self.regs.a);
    }
    fn cmc(&mut self, _mem: &mut [u8]) {
        let c = !self.regs.f.get_carry();
        self.regs.f.set_carry(c);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(CMC);
        #[cfg(feature = "log")]
        debug!("CMC {}", self.regs.f.get_carry());
    }
    fn stc(&mut self, _mem: &mut [u8]) {
        self.regs.f.set_carry(true);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(STC);
        #[cfg(feature = "log")]
        debug!("STC");
    }
    fn c_ccc(&mut self, mem: &mut [u8]) {
        let mut addr = 0;
        if self.regs.cond(self.instruction) {
            mem[self.regs.sp as usize - 1] = (self.regs.pc >> 8) as u8;
            mem[self.regs.sp as usize - 2] = self.regs.pc as u8;
            self.regs.sp -= 2;
            addr = self.get_16(mem);
            self.regs.pc = addr;
        } else {
            self.regs.pc += 3;
        }
        #[cfg(feature = "symlog")]
        log!(Cccc);
        #[cfg(feature = "log")]
        debug!("Cccc {:04X}", addr);
    }
    pub fn ret(&mut self, mem: &mut [u8]) {
        let addr = self.pop_16(mem);
        self.regs.pc = addr + 3;
        #[cfg(feature = "symlog")]
        log!(RET);
        #[cfg(feature = "log")]
        debug!("RET {:04X}", addr);
    }
    fn r_ccc(&mut self, mem: &mut [u8]) {
        let mut addr = 0;
        if self.regs.cond(self.instruction) {
            addr = self.pop_16(mem);
            self.regs.pc = addr + 3;
        } else {
            self.regs.pc += 1;
        }
        #[cfg(feature = "symlog")]
        log!(Rccc);
        #[cfg(feature = "log")]
        debug!("Rccc {:04X}", addr);
    }
    pub fn rst(&mut self, mem: &mut [u8]) {
        mem[self.regs.sp as usize - 1] = (self.regs.pc >> 8) as u8;
        mem[self.regs.sp as usize - 2] = self.regs.pc as u8;
        self.regs.sp -= 2;
        let addr = self.instruction & 0b00111000;
        self.regs.pc = addr as u16;
        #[cfg(feature = "log")]
        debug!("RST {:02X}", addr);
    }
    fn pchl(&mut self, _mem: &mut [u8]) {
        #[cfg(feature = "symlog")]
        log!(PCHL);
        #[cfg(feature = "log")]
        debug!("PCHL {:04x}", self.regs.pc);
        self.regs.pc = self.regs.get_rp(0x20);
    }
    fn pop(&mut self, mem: &mut [u8]) {
        let val = self.pop_16(mem);
        self.regs.set_rp(val, self.instruction);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(POP);
        #[cfg(feature = "log")]
        debug!("POP {:04x}", val);
    }
    fn xthl(&mut self, mem: &mut [u8]) {
        let l = self.regs.l;
        let h = self.regs.h;
        self.regs.l = mem[self.regs.sp as usize];
        self.regs.h = mem[self.regs.sp as usize + 1];
        mem[self.regs.sp as usize] = l;
        mem[self.regs.sp as usize + 1] = h;
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(XTHL);
        #[cfg(feature = "log")]
        debug!("XTHL {:04x}", self.regs.get_rp(0x20));
    }
    fn sphl(&mut self, _mem: &mut [u8]) {
        self.regs.sp = self.regs.get_rp(0x20);
        self.regs.pc += 1;
        #[cfg(feature = "symlog")]
        log!(SPHL);
        #[cfg(feature = "log")]
        debug!("SPHL {:04x}", self.regs.get_rp(0x20));
    }
    fn r#in(&mut self, mem: &mut [u8]) {
        let addr = mem[self.regs.pc as usize+1];
        let acc = self.input[addr as usize];
        self.regs.set_s(7, mem, acc);
        self.regs.pc +=2;
        #[cfg(feature = "log")]
        error!("IN {:02X}",acc);
        #[cfg(feature = "std")]
        panic!("IN at addr {:02X}", addr);
    }
    fn out(&mut self, mem: &mut [u8]) {
        let acc = self.regs.a;
        let addr = mem[self.regs.pc as usize+1];
        self.out_strobe = (true, addr, acc); 
        self.regs.pc +=2;
        #[cfg(feature = "symlog")]
        log!(OUT);
        #[cfg(feature = "log")]
        debug!("OUT {:02X}", acc);
    }
    fn ei(&mut self, _mem: &mut [u8]) {
        self.interrupt_enabled = true;
        self.regs.pc+=1;
        #[cfg(feature = "log")]
        debug!("EI");
        #[cfg(feature = "std")]
        panic!("EI at addr {:04X}", self.regs.pc);
    }
    fn di(&mut self, _mem: &mut [u8]) {
        self.interrupt_enabled = false;
        self.regs.pc+=1;
        #[cfg(feature = "log")]
        debug!("DI");
        #[cfg(feature = "std")]
        panic!("DI at addr {:04X}", self.regs.pc);
    }
    fn hlt(&mut self, _mem: &mut [u8]) {
        #[cfg(feature = "log")]
        error!("HLT");
        //#[cfg(feature = "std")]
        panic!("HLT at addr {:04X}", self.regs.pc);
    }
}
const INDEX: [(&str, fn(&mut CPU, &mut [u8])); 57] = [
    ("01DDDSSS", CPU::mov),
    ("00DDD110", CPU::mvi),
    ("00RP0001", CPU::lxi),
    ("00RP1010", CPU::ldax),
    ("00RP0010", CPU::stax),
    ("00111010", CPU::lda),
    ("00110010", CPU::sda),
    ("00101010", CPU::lhld),
    ("00100010", CPU::shld),
    ("11101011", CPU::xchg),
    ("10000SSS", CPU::add),
    ("11000110", CPU::adi),
    ("10001SSS", CPU::adc),
    ("11001110", CPU::aci),
    ("10010SSS", CPU::sub),
    ("11010110", CPU::sui),
    ("10011SSS", CPU::sbb),
    ("11011110", CPU::sbi),
    ("00DDD100", CPU::inr),
    ("00DDD101", CPU::dcr),
    ("00RP0011", CPU::inx),
    ("00RP1011", CPU::dcx),
    ("00RP1001", CPU::dad),
    ("00100111", CPU::daa),
    ("10100SSS", CPU::ana),
    ("11100110", CPU::ani),
    ("10110SSS", CPU::ora),
    ("11110110", CPU::ori),
    ("10101SSS", CPU::xra),
    ("11101110", CPU::xri),
    ("10111SSS", CPU::cmp),
    ("11111110", CPU::cpi),
    ("00000111", CPU::rlc),
    ("00001111", CPU::rrc),
    ("00010111", CPU::ral),
    ("00011111", CPU::rar),
    ("00101111", CPU::cma),
    ("00111111", CPU::cmc),
    ("00110111", CPU::stc),
    ("1100N011", CPU::jmp),
    ("11CCC010", CPU::jccc),
    ("11NN1101", CPU::call),
    ("11CCC100", CPU::c_ccc),
    ("110N1001", CPU::ret),
    ("11CCC000", CPU::r_ccc),
    ("11NNN111", CPU::rst),
    ("11101001", CPU::pchl),
    ("11RP0101", CPU::push),
    ("11RP0001", CPU::pop),
    ("11100011", CPU::xthl),
    ("11111001", CPU::sphl),
    ("11011011", CPU::r#in),
    ("11010011", CPU::out),
    ("11111011", CPU::ei),
    ("11110011", CPU::di),
    ("01110110", CPU::hlt),
    ("00NNN000", CPU::nop),
];
const LUT: [fn(&mut CPU, &mut [u8]); 0x100] = index();
const fn recursive(
    lut: &mut [fn(&mut CPU, &mut [u8]); 0x100],
    kmask: u8,
    xmask: u8,
    val: fn(&mut CPU, &mut [u8]),
) {
    if xmask == 0 {
        //debug!("kmask:{:03X}", kmask);
        lut[kmask as usize] = val;
    } else {
        let xmask_lsb = getLsb!(xmask);
        let xmask_without_lsb = xmask & !xmask_lsb;
        recursive(lut, kmask, xmask_without_lsb, val);
        recursive(lut, kmask | xmask_lsb, xmask_without_lsb, val);
    }
}
const fn place(lut: &mut [fn(&mut CPU, &mut [u8]); 0x100], s: &str, v: fn(&mut CPU, &mut [u8])) {
    let mut xmask: u8 = 0;
    let mut kmask: u8 = 0;
    let mut i = 0;
    let s = s.as_bytes();
    while i < s.len() {
        let c = s[i] as char;
        match c {
            '0' => {}
            '1' => kmask |= 1 << (7 - i),
            'N' | 'D' | 'R' | 'P' | 'S' | 'C' => xmask |= 1 << (7 - i),
            _ => {}
        }
        i = i + 1;
    }
    recursive(lut, kmask, xmask, v);
}

const fn index() -> [fn(&mut CPU, &mut [u8]); 0x100] {
    let mut lut: [fn(&mut CPU, &mut [u8]); 0x100] = [CPU::fault; 0x100];
    let mut i = 0;
    while i < INDEX.len() {
        place(&mut lut, INDEX[i].0, INDEX[i].1);
        i = i + 1;
    }
    return lut;
}
