pub mod flags;
use flags::Flags;
#[derive(Clone,Copy,Debug,Default)]
pub struct Registers{
    pub a:u8,
    pub b:u8,
    pub c:u8,
    pub d:u8,
    pub e:u8,
    pub h:u8,
    pub l:u8,
    pub sp:u16,
    pub pc:u16,
    pub f:Flags,
}
impl Registers{
    pub fn set_rp(&mut self,val:u16,instr:u8){
        match (instr & 0x30)>>4{
            0b00 => {self.b = (val >> 8) as u8;
                    self.c = val as u8},
            0b01 => {self.d = (val >> 8) as u8;
                    self.e = val as u8},
            0b10 => {self.h = (val >> 8) as u8;
                    self.l = val as u8},
            0b11 => if instr == 0xF5 || instr == 0xF1{
                self.a = (val >> 8) as u8;
                self.f.set(val as u8);
            }else{
                self.sp = val;
            },
            _=>{},
        }
    }
    pub fn get_rp(&self,instr:u8)->u16{
        match (instr & 0x30)>>4{
            0b00 => {(self.b as u16) << 8 | self.c as u16},
            0b01 => {(self.d as u16) << 8 | self.e as u16},
            0b10 => {(self.h as u16) << 8 | self.l as u16},
            0b11 => if instr == 0xF5 || instr == 0xF1{
                (self.a as u16) << 8 | self.f.get() as u16
            }else{
                self.sp
            },
            _=> {panic!()},
        }
    }
    pub fn get_d(&mut self,i:u8,mem:&[u8])->u8{
        match (i & 0x38) >> 3{
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => mem[self.get_rp(0x20) as usize],
            7 => self.a,
            _ =>  {panic!()}
        }
    }
    pub fn set_d(&mut self,i:u8,mem:&mut [u8],val:u8){
        match (i & 0x38) >> 3{
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => mem[self.get_rp(0x20) as usize] = val,
            7 => self.a = val,
            _ => {}
        };
    }
    pub fn get_s(&mut self,i:u8,mem:&[u8])->u8{
        match i & 7{
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => mem[self.get_rp(0x20) as usize],
            7 => self.a,
            _ =>  {panic!()}
        }
    }
    pub fn set_s(&mut self,i:u8,mem:&mut [u8],val:u8){
        match i & 7{
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => mem[self.get_rp(0x20) as usize] = val,
            7 => self.a = val,
            _ => {}
        };
    }
    pub fn set_flags(&mut self,r:u8,c:bool,h:bool){
        self.f.set_aux(h);
        self.f.set_carry(c);
        self.f.set_zero(r == 0);
        self.f.set_parity(r.count_ones() & 1 == 0);
        self.f.set_sign(r & 0x80 == 0x80);
    }
    pub fn cond(&mut self,i:u8)->bool{
        match (i & 0x38) >> 3{
            0 => !self.f.get_zero(),
            1 => self.f.get_zero(),
            2 => !self.f.get_carry(),
            3 => self.f.get_carry(),
            4 => !self.f.get_pairity(),
            5 => self.f.get_pairity(),
            6 => !self.f.get_sign(),
            7 => self.f.get_sign(),
            _ =>  {panic!()},
        }
    }
}