mod alu;
mod op;

use alu::*;
use op::*;

#[no_mangle]
pub extern "C" fn run(env: *const Env) -> i32 {
    let mut alu = Alu::new(env);

    alu.set_op(Op::Sra);
    alu.set_lhs(0xdeadbeef);
    alu.set_rhs(36);
    alu.eval();
    println!("res: 0x{:08x}", alu.res());

    alu.final_();

    0
}
