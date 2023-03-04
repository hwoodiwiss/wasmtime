use crate::machinst::{Reg, RegClass};
pub(crate) struct RegisterMapper;

impl crate::isa::unwind::winaarch64::RegisterMapper<Reg> for RegisterMapper {
    fn map(reg: Reg) -> crate::isa::unwind::winaarch64::MappedRegister {
        use crate::isa::unwind::winaarch64::MappedRegister;
        match reg.class() {
            // Map Registers from RegClass -> MappedRegister
            //x64 example RegClass::Int => MappedRegister::Int(reg.to_real_reg().unwrap().hw_enc()),
            RegClass::Int => unimplemented!(""),
            regalloc2::RegClass::Float => unimplemented!("")
        }
    }
}