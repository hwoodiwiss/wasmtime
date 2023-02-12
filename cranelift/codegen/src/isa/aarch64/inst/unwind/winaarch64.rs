use crate::machinst::{Reg};
pub(crate) struct RegisterMapper;

impl crate::isa::unwind::winaarch64::RegisterMapper<Reg> for RegisterMapper {
    fn map(&self, reg: Reg) -> Result<u16, RegisterMappingError> {
        Ok(map_reg(reg)?.0)
    }
}