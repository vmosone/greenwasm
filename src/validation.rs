use super::structure::types::FuncType;
use super::structure::types::TableType;
use super::structure::types::MemType;
use super::structure::types::GlobalType;
use super::structure::types::ValType;
use super::structure::types::ResultType;
use super::structure::types::Limits;

use super::structure::instructions::Instr;

#[derive(Default)]
pub struct Ctx<'a> {
    prepended_to: Option<&'a Ctx<'a>>,

    types: Vec<FuncType>,
    funcs: Vec<FuncType>,
    tables: Vec<TableType>,
    mems: Vec<MemType>,
    globals: Vec<GlobalType>,
    locals: Vec<ValType>,
    labels: Vec<ResultType>,
    return_: Option<ResultType>,
}

pub type VResult = Result<(), ValidationError>;
pub struct ValidationError {
    pub kind: ValidationErrorEnum,
}
use self::ValidationErrorEnum::*;

impl<'a> Ctx<'a> {
    fn error(&self, error: ValidationErrorEnum) -> VResult {
        Err(ValidationError {
            kind: error
        })
    }

    fn ok(&self) -> VResult {
        Ok(())
    }
}

pub enum ValidationErrorEnum {
    LimitMaxSmallerMin,
    FunctionTypeResultArityGreaterOne,
    InstrSetGlobalNotVar,
    InstrLoadOveraligned,
    InstrStoreOveraligned,
    InstrBrTableNotSameLabelType,
    InstrCallIndirectElemTypeNotAnyFunc,
}

impl<'a> Ctx<'a> {
    fn local(&self, x: u32) -> Result<ValType, ValidationError> {
        unimplemented!()
    }
    fn global(&self, x: u32) -> Result<GlobalType, ValidationError> {
        unimplemented!()
    }
    fn mem(&self, x: u32) -> Result<MemType, ValidationError> {
        unimplemented!()
    }
    fn label(&self, x: u32) -> Result<ResultType, ValidationError> {
        unimplemented!()
    }
    fn return_(&self) -> Result<ResultType, ValidationError> {
        unimplemented!()
    }
    fn func(&self, x: u32) -> Result<FuncType, ValidationError> {
        unimplemented!()
    }
    fn table(&self, x: u32) -> Result<TableType, ValidationError> {
        unimplemented!()
    }
    fn type_(&self, x: u32) -> Result<FuncType, ValidationError> {
        unimplemented!()
    }
    fn with_prepended_label(&'a self, resulttype: ResultType) -> Ctx<'a> {
        Ctx {
            prepended_to: Some(self),
            labels: vec![resulttype],
            ..Default::default()
        }
    }
}

macro_rules! valid {
    ($self:ident, $name:ident: $type:ty, $b:block) => (
        pub fn $name(&$self, $name: &$type) -> VResult {
            $b
            $self.ok()
        }
    )
}

#[derive(Eq, PartialEq)]
enum AnyValType {
    I32,
    I64,
    F32,
    F64,
    Any(char),
    AnySeq,
    None,
}

trait AnyValTypeBuilder {
    fn append()
}

fn filter(v: Vec<AnyValType>) -> Vec<AnyValType> {
    v.into_iter().filter(|x| *x != AnyValType::None).collect()
}

impl From<ValType> for AnyValType {
    fn from(other: ValType) -> Self {
        match other {
            ValType::I32 => AnyValType::I32,
            ValType::I64 => AnyValType::I64,
            ValType::F32 => AnyValType::F32,
            ValType::F64 => AnyValType::F64,
        }
    }
}
impl From<ResultType> for AnyValType {
    fn from(other: ResultType) -> Self {
        match other {
            Some(t) => t.into(),
            None => AnyValType::None,
        }
    }
}

fn any(t: char) -> AnyValType {
    AnyValType::Any(t)
}

fn any_seq() -> AnyValType {
    AnyValType::AnySeq
}

pub struct AnyFuncType {
    args: Vec<AnyValType>,
    results: Vec<AnyValType>,
}
impl From<FuncType> for AnyFuncType {
    fn from(FuncType { args, results }: FuncType) -> Self {
        AnyFuncType {
            args: args.into_iter().map(|x| x.into()).collect(),
            results: results.into_iter().map(|x| x.into()).collect(),
        }
    }
}

macro_rules! ty {
    ($($a:expr),*;$($r:expr),*) => (AnyFuncType {
        args: filter(vec![$($a.into()),*]),
        results: filter(vec![$($r.into()),*]),
    })
}

macro_rules! valid_with {
    ($self:ident, $name:ident: $type:ty, $valid_ty:ident, $b:block) => (
        pub fn $name(&$self, $name: &$type, $valid_ty: &AnyFuncType) -> VResult {
            $b
            $self.ok()
        }
    )
}

impl<'a> Ctx<'a> {
    valid!(self, limit: Limits, {
        if limit.max.unwrap_or(0) < limit.min {
            self.error(LimitMaxSmallerMin)?
        }
    });

    valid!(self, function_type: FuncType, {
        if function_type.results.len() > 1 {
            self.error(FunctionTypeResultArityGreaterOne)?
        }
    });

    valid!(self, table_type: TableType, {
        self.limit(&table_type.limits)?;
    });

    valid!(self, memory_type: MemType, {
        self.limit(&memory_type.limits)?;
    });

    valid!(self, _global_types: GlobalType, {
    });

    valid_with!(self, instruction: Instr, valid_ty, {
        use self::Instr::*;
        use self::ValType::*;

        let ty = match *instruction {
            // numeric instructions
            TConst(x)       => ty![               ; x.ty()],
            TUnop(x)        => ty![x.ty()         ; x.ty()],
            TBinop(x)       => ty![x.ty(), x.ty() ; x.ty()],
            IxxTestop(x, _) => ty![x.ty()         ; I32   ],
            TRelop(x)       => ty![x.ty(), x.ty() ; x.ty()],
            // cvtops
            TReinterpret(t)          => ty![t.from_ty() ; t.ty() ],
            IxxTruncFxx(t2, _, t1)   => ty![t1.ty()     ; t2.ty()],
            I32WrapI64               => ty![I64         ; I32    ],
            I64ExtendI32(_)          => ty![I32         ; I64    ],
            FxxConvertUxx(t2, _, t1) => ty![t1.ty()     ; t2.ty()],
            F32DemoteF64             => ty![F64         ; F32    ],
            F64PromoteF32            => ty![F32         ; F64    ],

            // parametric instructions
            Drop   => ty![any('t')                ;         ],
            Select => ty![any('t'), any('t'), I32 ; any('t')],

            // variable instructions
            GetLocal(x) => {
                let t = self.local(x)?;
                ty![ ; t]
            }
            SetLocal(x) => {
                let t = self.local(x)?;
                ty![t ; ]
            }
            TeeLocal(x) => {
                let t = self.local(x)?;
                ty![t ; t]
            }
            GetGlobal(x) => {
                let mut_t = self.global(x)?;
                let t = mut_t.valtype;
                ty![ ; t]
            }
            SetGlobal(x) => {
                let mut_t = self.global(x)?;
                let t = mut_t.valtype;
                use super::structure::types::Mut;
                if mut_t.mutability != Mut::Var  {
                    self.error(InstrSetGlobalNotVar)?;
                }
                ty![t ; ]
            }

            // memory instructions
            ref load_store_instr @ TLoad(..) |
            ref load_store_instr @ IxxLoad8(..) |
            ref load_store_instr @ IxxLoad16(..) |
            ref load_store_instr @ I64Load32(..) |
            ref load_store_instr @ TStore(..) |
            ref load_store_instr @ IxxStore8(..) |
            ref load_store_instr @ IxxStore16(..) |
            ref load_store_instr @ I64Store32(..) => {
                use super::structure::instructions::Memarg;

                let validate = |t: ValType, memarg: Memarg, bit_width, e, r| {
                    self.mem(0)?;
                    let align = 1u32 << memarg.align;
                    if align > (bit_width / 8) {
                        self.error(e)?;
                    }
                    Ok(r)
                };
                let load = |t: ValType, memarg: Memarg, bit_width| {
                    validate(t, memarg, bit_width,
                             InstrLoadOveraligned, ty![I32 ; t])
                };
                let store = |t: ValType, memarg: Memarg, bit_width| {
                    validate(t, memarg, bit_width,
                             InstrStoreOveraligned, ty![I32, t ; ])
                };

                match *load_store_instr {
                    TLoad(t, memarg)        => load(t, memarg, t.bit_width())?,
                    IxxLoad8(t, _, memarg)  => load(t.ty(), memarg, 8)?,
                    IxxLoad16(t, _, memarg) => load(t.ty(), memarg, 16)?,
                    I64Load32(_, memarg)    => load(I64, memarg, 32)?,

                    TStore(t, memarg)       => store(t, memarg, t.bit_width())?,
                    IxxStore8(t, memarg)    => store(t.ty(), memarg, 8)?,
                    IxxStore16(t, memarg)   => store(t.ty(), memarg, 16)?,
                    I64Store32(memarg)      => store(I64, memarg, 32)?,

                    _ => unreachable!(),
                }
            }
            CurrentMemory => { self.mem(0)?; ty![    ; I32] }
            GrowMemory    => { self.mem(0)?; ty![I32 ; I32] }

            // control instructions
            Nop => ty![ ; ],
            Unreachable => ty![any_seq() ; any_seq()],
            Block(resulttype, ref block) => {
                let self_ = self.with_prepended_label(resulttype);
                let ty = ty![ ; resulttype];
                self_.instruction_sequence(block, &ty)?;
                ty
            }
            Loop(resulttype, ref block) => {
                let self_ = self.with_prepended_label(None);
                let ty = ty![ ; resulttype];
                self_.instruction_sequence(block, &ty)?;
                ty
            }
            IfElse(resulttype, ref if_block, ref else_block) => {
                let self_ = self.with_prepended_label(resulttype);
                let ty = ty![ ; resulttype];
                self_.instruction_sequence(if_block, &ty)?;
                self_.instruction_sequence(else_block, &ty)?;
                ty![I32 ; resulttype]
            }
            Br(labelidx) => {
                let resulttype = self.label(labelidx)?;
                ty![any_seq(), resulttype ; any_seq()]
            }
            BrIf(labelidx) => {
                let resulttype = self.label(labelidx)?;
                ty![resulttype, I32; resulttype]
            }
            BrTable(ref labelindices, labelidx_n) => {
                let resulttype = self.label(labelidx_n)?;

                for &li in labelindices {
                    let resulttype_i = self.label(li)?;
                    if resulttype_i != resulttype {
                        self.error(InstrBrTableNotSameLabelType)?;
                    }
                }

                ty![any_seq(), resulttype, I32 ; any_seq()]
            }
            Return => {
                let resulttype = self.return_()?;
                ty![any_seq(), resulttype ; any_seq()]
            }
            Call(x) => {
                self.func(x)?.into()
            }
            CallIndirect(x) => {
                let TableType {
                    limits,
                    elemtype,
                } = self.table(0)?;
                use super::structure::types::ElemType;
                if elemtype != ElemType::AnyFunc {
                    self.error(InstrCallIndirectElemTypeNotAnyFunc)?;
                }
                let ty = self.type_(x)?;
                ty![ty.args, I32 ; ty.result]
            }


            _ => unimplemented!(),
        };

        // TODO: What to do with type?

        unimplemented!()
    });

    valid_with!(self, instruction_sequence: [Instr], valid_ty, {

    });
}
