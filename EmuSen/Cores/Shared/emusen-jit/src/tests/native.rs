//! Version 1.1's tailored code: published in place of the function, its stubs reached, its fallback to the function.

use cranelift_codegen::ir::types::I64;
use cranelift_codegen::ir::{self, AbiParam, InstBuilder};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::JITModule;
use cranelift_module::Module;
use emusen_assembler::x64::{self, X64};
use emusen_assembler::{Assembler, Cond, Section, Src, Width};

use crate::native::Native;
use crate::{Jit, Lowering};

/// A lowering whose function answers 7 and whose tailored code answers its word plus one through a stub, or jumps to the function where its argument is 0.
#[derive(Clone)]
struct Two;

impl Lowering for Two {
    type Options = bool;
    const NAME: &'static str = "two";
    const EXITS: &'static [&'static str] = &[];

    fn lower(&mut self, module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, _: &[u32], _: u32, _: bool) -> Result<(), String> {
        let ptr = module.target_config().pointer_type();
        func.signature.params.push(AbiParam::new(ptr));
        func.signature.returns.push(AbiParam::new(I64));
        let mut b = FunctionBuilder::new(func, functions);
        let e = b.create_block();
        b.append_block_params_for_function_params(e);
        b.switch_to_block(e);
        b.seal_block(e);
        let v = b.ins().iconst(I64, 7);
        b.ins().return_(&[v]);
        b.finalize(module.target_config());
        Ok(())
    }

    fn native(&mut self, words: &[u32], _: u32, tailored: bool) -> Option<Native> {
        if !tailored {
            return None;
        }
        let mut a = X64::new();
        let entry = a.label();
        let slow = a.label();
        let slot = a.data(&[0; 8]);
        a.bind(entry);
        a.cmp(Width::W64, x64::RDI, Src::Imm(0));
        a.branch(Cond::Eq, slow);
        a.mov_imm(x64::RAX, i64::from(words[0]));
        a.jump_stub(0);
        a.section(Section::Cold);
        a.bind(slow);
        a.jump_via_data(slot);
        Some(Native { code: a.finish(entry), fallback: Some(slot) })
    }

    fn stubs(&self) -> Vec<emusen_assembler::Code> {
        let mut a = X64::new();
        let e = a.label();
        a.bind(e);
        a.alu(emusen_assembler::Op::Add, Width::W64, x64::RAX, x64::RAX, Src::Imm(1));
        a.ret();
        vec![a.finish(e)]
    }
}

#[test]
fn tailored_code_is_published_in_place_of_the_function_and_falls_back_to_it() {
    let mut jit = Jit::new(Two, 1 << 20);
    let plain = jit.enqueue(vec![5].into_boxed_slice(), 0, false);
    let slots: Vec<_> = (0..300u32).map(|k| jit.enqueue(vec![k * 3].into_boxed_slice(), 0, true)).collect();
    type F = extern "C" fn(u64) -> u64;
    // SAFETY: each published address is a function or tailored code of this signature.
    let call = |p: *const u8, x: u64| unsafe { std::mem::transmute::<*const u8, F>(p)(x) };
    assert_eq!(call(plain.wait().expect("published"), 1), 7);
    for (k, s) in slots.iter().enumerate() {
        let p = s.wait().expect("published");
        assert_eq!(call(p, 1), k as u64 * 3 + 1, "the tailored code and the stub");
        assert_eq!(call(p, 0), 7, "the fallback to the function");
    }
    let c = jit.counters().expect("running");
    assert_eq!(c.native.load(std::sync::atomic::Ordering::Relaxed), 300);
}
