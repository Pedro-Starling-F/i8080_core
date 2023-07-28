#[derive(Copy, Clone, Debug, Default)]
pub struct Flags(u8);
impl Flags {
    pub fn set(&mut self,s:u8){
        self.0 = s;
    }
    pub fn get(&self) ->u8{
        self.0
    }
    fn sets(&mut self,val:bool,shift:u8){
        let val = (val as u8) << shift;
        let mask = 1 << shift;
        self.0&=!mask;
        self.0|=val;
    }
    pub fn set_sign(&mut self, val: bool) {
        self.sets(val,7);
    }
    pub fn set_zero(&mut self, val: bool) { 
        self.sets(val,6);
    }
    pub fn set_aux(&mut self, val: bool) {
        self.sets(val,4);
    }
    pub fn set_parity(&mut self, val: bool) {
        self.sets(val,2);
    }
    pub  fn set_carry(&mut self, val: bool) {
        self.sets(val,0);
    }
    fn gets(self,shift:u8) ->bool{
        let val = self.0 >> shift;
        val&1 == 1
    }
    pub fn get_sign(&self) -> bool{
        self.gets(7)
    }
    pub fn get_zero(&self) -> bool{
        self.gets(6)
    }
    pub fn get_aux(&self) -> bool{
        self.gets(4)
    }
    pub fn get_pairity(&self) -> bool{
        self.gets(2)
    }
    pub fn get_carry(&self) -> bool{
        self.gets(0)
    }
}