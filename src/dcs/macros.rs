macro_rules! dcs_basic_command {
    ($instr_name:ident,$instr:expr) => {
        ///
        /// Basic command implementation for any instruction with no parameters
        ///
        pub struct $instr_name;

        impl DcsCommand for $instr_name {
            fn instruction(&self) -> Instruction {
                $instr
            }

            fn fill_params_buf(&self, _buffer: &mut [u8]) -> Result<usize, Error> {
                Ok(0)
            }
        }
    };
}
