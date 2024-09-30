use std::{collections::HashMap, error::Error, path::PathBuf};

use cairo_lang_sierra::{
    extensions::{
        array::ArrayConcreteLibfunc,
        const_type::ConstConcreteLibfunc,
        core::{CoreConcreteLibfunc, CoreLibfunc, CoreType, CoreTypeConcrete},
        enm::EnumConcreteLibfunc,
        gas::GasConcreteLibfunc,
        int::{unsigned::UintConcrete, IntOperator},
        mem::MemConcreteLibfunc,
        structure::StructConcreteLibfunc,
        ConcreteLibfunc,
    },
    ids::{ConcreteTypeId, VarId},
    program::{GenStatement, GenericArg, StatementIdx},
    program_registry::ProgramRegistry,
    ProgramParser,
};
use clap::Parser;

/// Compiles a Cairo project outputting the generated MLIR and the shared library.
/// Exits with 1 if the compilation or run fails, otherwise 0.
#[derive(Parser, Debug)]
#[clap(version, verbatim_doc_comment)]
struct Args {
    /// The path to the sierra file.
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let program_src = std::fs::read_to_string(&args.path)?;

    let program_parser = ProgramParser::new();

    let mut buf = String::new();

    let program = program_parser
        .parse(&program_src)
        .map_err(|e| e.map_token(|t| t.to_string()))?;

    let reg: ProgramRegistry<CoreType, CoreLibfunc> = ProgramRegistry::new(&program)?;

    buf.push('\n');

    for func in &program.funcs {
        let mut buffunc = String::new();

        buffunc.push_str(&format!("pub fn func_{}(", func.id.id));

        let mut first = true;
        for (i, param) in func.signature.param_types.iter().enumerate() {
            let ty_name = get_type_name(&reg, param)?;

            if first {
                buffunc.push_str(&format!("v{i}: {ty_name}"));
                first = false;
            } else {
                buffunc.push_str(&format!(", v{i}: {ty_name}"));
            }
        }

        buffunc.push_str(") -> (");

        first = true;
        for param in func.signature.ret_types.iter() {
            let ty_name = get_type_name(&reg, param)?;

            if first {
                buffunc.push_str(&ty_name.to_string());
                first = false;
            } else {
                buffunc.push_str(&format!(", {ty_name}"));
            }
        }

        buffunc.push_str(") {\n");

        let mut varids: HashMap<VarId, ()> = HashMap::new();

        build_statement(
            &mut buffunc,
            &reg,
            func.entry_point,
            &program.statements,
            &mut varids,
            1,
        )?;

        buffunc.push_str("}\n\n");

        buf.push_str(&buffunc);
    }

    std::fs::write("out.cairo_dec", buf)?;
    Ok(())
}

pub fn build_statement(
    buffunc: &mut String,
    reg: &ProgramRegistry<CoreType, CoreLibfunc>,
    statement_idx: StatementIdx,
    statements: &[GenStatement<StatementIdx>],
    varids: &mut HashMap<VarId, ()>,
    depth_level: usize,
) -> Result<(), Box<dyn Error>> {
    let st = &statements[statement_idx.0];
    let tabs: String = " ".chars().cycle().take(depth_level * 4).collect();

    match st {
        GenStatement::Invocation(gen_invocation) => {
            let lb = reg.get_libfunc(&gen_invocation.libfunc_id)?;

            for b in &gen_invocation.branches {
                for v in &b.results {
                    varids.insert(v.clone(), ());
                }
            }

            match lb {
                CoreConcreteLibfunc::ApTracking(_) => {
                    assert_eq!(gen_invocation.branches.len(), 1);
                    let idx = statement_idx.next(&gen_invocation.branches[0].target);
                    build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                }
                CoreConcreteLibfunc::Array(selector) => match selector {
                    ArrayConcreteLibfunc::New(info) => {
                        let outvarid = &gen_invocation.branches[0].results[0];
                        let out_ty = &info.branch_signatures()[0].vars[0].ty;

                        buffunc.push_str(&format!(
                            "{tabs}let mut v{:?}: {} = Array::new();\n",
                            outvarid.id,
                            get_type_name(reg, out_ty)?
                        ));

                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                    ArrayConcreteLibfunc::SpanFromTuple(_) => todo!(),
                    ArrayConcreteLibfunc::TupleFromSpan(_) => todo!(),
                    ArrayConcreteLibfunc::Append(info) => {
                        let outvarid = &gen_invocation.branches[0].results[0];
                        let out_ty = &info.branch_signatures()[0].vars[0].ty;

                        buffunc.push_str(&format!(
                            "{tabs}v{:?}.append(v{:?});\n",
                            gen_invocation.args[0].id, gen_invocation.args[1].id,
                        ));

                        buffunc.push_str(&format!(
                            "{tabs}let mut v{:?}: {} = v{:?};\n",
                            outvarid.id,
                            get_type_name(reg, out_ty)?,
                            gen_invocation.args[0].id,
                        ));

                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                    ArrayConcreteLibfunc::PopFront(_) => todo!(),
                    ArrayConcreteLibfunc::PopFrontConsume(_) => todo!(),
                    ArrayConcreteLibfunc::Get(_) => todo!(),
                    ArrayConcreteLibfunc::Slice(_) => todo!(),
                    ArrayConcreteLibfunc::Len(_) => todo!(),
                    ArrayConcreteLibfunc::SnapshotPopFront(_) => todo!(),
                    ArrayConcreteLibfunc::SnapshotPopBack(_) => todo!(),
                    ArrayConcreteLibfunc::SnapshotMultiPopFront(_) => todo!(),
                    ArrayConcreteLibfunc::SnapshotMultiPopBack(_) => todo!(),
                },
                CoreConcreteLibfunc::BranchAlign(_) => {
                    assert_eq!(gen_invocation.branches.len(), 1);
                    let idx = statement_idx.next(&gen_invocation.branches[0].target);
                    build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                }
                CoreConcreteLibfunc::Bool(_) => todo!(),
                CoreConcreteLibfunc::Box(_) => todo!(),
                CoreConcreteLibfunc::Cast(_) => todo!(),
                CoreConcreteLibfunc::Circuit(_) => todo!(),
                CoreConcreteLibfunc::Coupon(_) => todo!(),
                CoreConcreteLibfunc::CouponCall(_) => todo!(),
                CoreConcreteLibfunc::Drop(_) => {
                    let args = &gen_invocation.args;
                    buffunc.push_str(&format!("{tabs}drop(v{:?});\n", args[0].id,));

                    assert_eq!(gen_invocation.branches.len(), 1);
                    let idx = statement_idx.next(&gen_invocation.branches[0].target);
                    build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                }
                CoreConcreteLibfunc::Dup(_info) => {
                    todo!()
                }
                CoreConcreteLibfunc::Ec(_) => todo!(),
                CoreConcreteLibfunc::Felt252(_) => todo!(),
                CoreConcreteLibfunc::Const(selector) => match selector {
                    ConstConcreteLibfunc::AsBox(_) => todo!(),
                    ConstConcreteLibfunc::AsImmediate(info) => {
                        let outvarid = &gen_invocation.branches[0].results[0];
                        let out_ty = get_type_name(reg, &info.branch_signatures()[0].vars[0].ty)?;
                        let const_type = reg.get_type(&info.const_type)?;

                        let out_val = if let CoreTypeConcrete::Const(info) = const_type {
                            let mut data = String::new();

                            for x in &info.inner_data {
                                match x {
                                    GenericArg::UserType(_user_type_id) => todo!(),
                                    GenericArg::Type(_concrete_type_id) => todo!(),
                                    GenericArg::Value(big_int) => {
                                        data.push_str(&big_int.to_str_radix(10))
                                    }
                                    GenericArg::UserFunc(_function_id) => todo!(),
                                    GenericArg::Libfunc(_concrete_libf_unc_id) => todo!(),
                                }
                            }

                            data
                        } else {
                            unreachable!()
                        };

                        buffunc.push_str(&format!(
                            "{tabs}let v{:?}: {out_ty} = {out_val};\n",
                            outvarid.id
                        ));

                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                },
                CoreConcreteLibfunc::FunctionCall(_) => todo!(),
                CoreConcreteLibfunc::Gas(selector) => match selector {
                    GasConcreteLibfunc::WithdrawGas(_) => {}
                    GasConcreteLibfunc::RedepositGas(_) => todo!(),
                    GasConcreteLibfunc::GetAvailableGas(_) => todo!(),
                    GasConcreteLibfunc::BuiltinWithdrawGas(_) => {}
                    GasConcreteLibfunc::GetBuiltinCosts(_) => todo!(),
                },
                CoreConcreteLibfunc::Uint8(_) => todo!(),
                CoreConcreteLibfunc::Uint16(_) => todo!(),
                CoreConcreteLibfunc::Uint32(selector) => match selector {
                    UintConcrete::Const(_) => todo!(),
                    UintConcrete::Operation(info) => {
                        let outvarid = &gen_invocation.branches[1].results[1];
                        let out_ty = &info.branch_signatures()[1].vars[1].ty;
                        let op = match info.operator {
                            IntOperator::OverflowingAdd => '+',
                            IntOperator::OverflowingSub => '-',
                        };

                        let lhs = &gen_invocation.args[1];
                        let rhs = &gen_invocation.args[2];

                        buffunc.push_str(&format!(
                            "{tabs}let (v{:?} : {}, v{:?}_overflowed: bool) = v{:?} {op} v{:?};\n",
                            outvarid.id,
                            get_type_name(reg, out_ty)?,
                            outvarid.id,
                            lhs.id,
                            rhs.id
                        ));

                        buffunc.push_str(&format!("{tabs}if !v{:?}_overflowed {{\n", outvarid.id));

                        {
                            let range_check_id = gen_invocation.branches[0].results[0].id;
                            let out_ty = &info.branch_signatures()[0].vars[0].ty;
                            buffunc.push_str(&format!(
                                "{tabs}    let v{}: {} = v{:?};\n",
                                range_check_id,
                                get_type_name(reg, out_ty)?,
                                gen_invocation.args[0].id
                            ));
                        }

                        assert_eq!(gen_invocation.branches.len(), 2);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level + 1)?;

                        buffunc.push_str(&format!("{tabs}}} else {{\n"));

                        {
                            let range_check_id = gen_invocation.branches[1].results[0].id;
                            let out_ty = &info.branch_signatures()[1].vars[0].ty;
                            buffunc.push_str(&format!(
                                "{tabs}    let v{}: {} = v{:?};\n",
                                range_check_id,
                                get_type_name(reg, out_ty)?,
                                gen_invocation.args[0].id
                            ));
                        }

                        let idx = statement_idx.next(&gen_invocation.branches[1].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level + 1)?;

                        buffunc.push_str(&format!("{tabs}}}\n"));
                    }
                    UintConcrete::SquareRoot(_) => todo!(),
                    UintConcrete::Equal(_) => todo!(),
                    UintConcrete::ToFelt252(_) => todo!(),
                    UintConcrete::FromFelt252(_) => todo!(),
                    UintConcrete::IsZero(_) => todo!(),
                    UintConcrete::Divmod(_) => todo!(),
                    UintConcrete::WideMul(_) => todo!(),
                    UintConcrete::Bitwise(_) => todo!(),
                },
                CoreConcreteLibfunc::Uint64(_) => todo!(),
                CoreConcreteLibfunc::Uint128(_) => todo!(),
                CoreConcreteLibfunc::Uint256(_) => todo!(),
                CoreConcreteLibfunc::Uint512(_) => todo!(),
                CoreConcreteLibfunc::Sint8(_) => todo!(),
                CoreConcreteLibfunc::Sint16(_) => todo!(),
                CoreConcreteLibfunc::Sint32(_) => todo!(),
                CoreConcreteLibfunc::Sint64(_) => todo!(),
                CoreConcreteLibfunc::Sint128(_) => todo!(),
                CoreConcreteLibfunc::Mem(selector) => match selector {
                    MemConcreteLibfunc::StoreTemp(_) => {
                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                    MemConcreteLibfunc::StoreLocal(_) => {
                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                    MemConcreteLibfunc::FinalizeLocals(_) => {
                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                    MemConcreteLibfunc::AllocLocal(_) => {
                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                    MemConcreteLibfunc::Rename(_) => todo!(),
                },
                CoreConcreteLibfunc::Nullable(_) => todo!(),
                CoreConcreteLibfunc::UnwrapNonZero(_) => todo!(),
                CoreConcreteLibfunc::UnconditionalJump(_) => todo!(),
                CoreConcreteLibfunc::Enum(selector) => {
                    match selector {
                        EnumConcreteLibfunc::Init(info) => {
                            let outvarid = &gen_invocation.branches[0].results[0];
                            let args = &gen_invocation.args;
                            let out_ty = &info.branch_signatures()[0].vars[0].ty;
                            let variant = info.index;

                            // todo: get variant type, use it as :: part

                            buffunc.push_str(&format!(
                                "{tabs}let v{:?}: {} = Enum::Variant{:?}(",
                                outvarid.id,
                                get_type_name(reg, out_ty)?,
                                variant
                            ));

                            let mut first = true;
                            for arg in args {
                                if first {
                                    buffunc.push_str(&format!("v{:?}", arg.id));
                                    first = false;
                                } else {
                                    buffunc.push_str(&format!(", v{:?}", arg.id));
                                }
                            }
                            buffunc.push_str(");\n");

                            assert_eq!(gen_invocation.branches.len(), 1);
                            let idx = statement_idx.next(&gen_invocation.branches[0].target);
                            build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                        }
                        EnumConcreteLibfunc::FromBoundedInt(_) => todo!(),
                        EnumConcreteLibfunc::Match(_) => todo!(),
                        EnumConcreteLibfunc::SnapshotMatch(_) => todo!(),
                    }
                }
                CoreConcreteLibfunc::Struct(selector) => match selector {
                    StructConcreteLibfunc::Construct(info) => {
                        let outvarid = &gen_invocation.branches[0].results[0];
                        let args = &gen_invocation.args;
                        let out_ty = &info.branch_signatures()[0].vars[0].ty;

                        buffunc.push_str(&format!(
                            "{tabs}let v{:?}: {} = Struct {{\n",
                            outvarid.id,
                            get_type_name(reg, out_ty)?
                        ));

                        for (field, arg) in args.iter().enumerate() {
                            buffunc.push_str(&format!("{tabs}\tfield_{field}: v{:?},\n", arg.id));
                        }
                        buffunc.push_str(&format!("{tabs}}};\n"));

                        assert_eq!(gen_invocation.branches.len(), 1);
                        let idx = statement_idx.next(&gen_invocation.branches[0].target);
                        build_statement(buffunc, reg, idx, statements, varids, depth_level)?;
                    }
                    StructConcreteLibfunc::Deconstruct(_) => todo!(),
                    StructConcreteLibfunc::SnapshotDeconstruct(_) => todo!(),
                },
                CoreConcreteLibfunc::Felt252Dict(_) => todo!(),
                CoreConcreteLibfunc::Felt252DictEntry(_) => todo!(),
                CoreConcreteLibfunc::Pedersen(_) => todo!(),
                CoreConcreteLibfunc::Poseidon(_) => todo!(),
                CoreConcreteLibfunc::StarkNet(_) => todo!(),
                CoreConcreteLibfunc::Debug(_) => todo!(),
                CoreConcreteLibfunc::SnapshotTake(_) => todo!(),
                CoreConcreteLibfunc::Bytes31(_) => todo!(),
                CoreConcreteLibfunc::BoundedInt(_) => todo!(),
            }
        }
        GenStatement::Return(vec) => {
            buffunc.push('\n');
            buffunc.push_str(&format!("{tabs}return "));

            let mut first = true;
            for id in vec {
                if first {
                    first = false;
                    buffunc.push_str(&format!("v{:?}", id.id));
                } else {
                    buffunc.push_str(&format!(", v{:?}", id.id));
                }
            }
            buffunc.push_str(";\n");

            return Ok(());
        }
    }

    Ok(())
}

pub fn get_type_name(
    reg: &ProgramRegistry<CoreType, CoreLibfunc>,
    type_id: &ConcreteTypeId,
) -> Result<String, Box<dyn Error>> {
    let ty = reg.get_type(type_id)?;

    Ok(match ty {
        CoreTypeConcrete::Array(inner) => {
            let inner_str = get_type_name(reg, &inner.ty)?;
            format!("Array<{}>", inner_str)
        }
        CoreTypeConcrete::Coupon(_) => todo!(),
        CoreTypeConcrete::Bitwise(_) => todo!(),
        CoreTypeConcrete::Box(_) => todo!(),
        CoreTypeConcrete::Circuit(_) => todo!(),
        CoreTypeConcrete::Const(info) => {
            let mut data = String::new();

            for x in &info.inner_data {
                match x {
                    GenericArg::UserType(_user_type_id) => todo!(),
                    GenericArg::Type(_concrete_type_id) => todo!(),
                    GenericArg::Value(big_int) => data.push_str(&big_int.to_str_radix(10)),
                    GenericArg::UserFunc(_function_id) => todo!(),
                    GenericArg::Libfunc(_concrete_libf_unc_id) => todo!(),
                }
            }

            data
        }
        CoreTypeConcrete::EcOp(_) => todo!(),
        CoreTypeConcrete::EcPoint(_) => todo!(),
        CoreTypeConcrete::EcState(_) => todo!(),
        CoreTypeConcrete::Felt252(_) => "felt252".to_string(),
        CoreTypeConcrete::GasBuiltin(_) => "GasBuiltin".to_string(),
        CoreTypeConcrete::BuiltinCosts(_) => "BuiltinCosts".to_string(),
        CoreTypeConcrete::Uint8(_) => "u8".to_string(),
        CoreTypeConcrete::Uint16(_) => "u16".to_string(),
        CoreTypeConcrete::Uint32(_) => "u32".to_string(),
        CoreTypeConcrete::Uint64(_) => "u64".to_string(),
        CoreTypeConcrete::Uint128(_) => "u128".to_string(),
        CoreTypeConcrete::Uint128MulGuarantee(_) => "Uint128MulGuarantee".to_string(),
        CoreTypeConcrete::Sint8(_) => "i8".to_string(),
        CoreTypeConcrete::Sint16(_) => "i16".to_string(),
        CoreTypeConcrete::Sint32(_) => "i32".to_string(),
        CoreTypeConcrete::Sint64(_) => "i64".to_string(),
        CoreTypeConcrete::Sint128(_) => "i128".to_string(),
        CoreTypeConcrete::NonZero(inner) => {
            let inner_str = get_type_name(reg, &inner.ty)?;
            format!("NonZero<{}>", inner_str)
        }
        CoreTypeConcrete::Nullable(inner) => {
            let inner_str = get_type_name(reg, &inner.ty)?;
            format!("Nullable<{}>", inner_str)
        }
        CoreTypeConcrete::RangeCheck(_) => "RangeCheck".to_string(),
        CoreTypeConcrete::RangeCheck96(_) => "RangeCheck96".to_string(),
        CoreTypeConcrete::Uninitialized(inner) => {
            let inner_str = get_type_name(reg, &inner.ty)?;
            format!("Uninitialized<{}>", inner_str)
        }
        CoreTypeConcrete::Enum(info) => {
            let mut buf = String::new();

            buf.push_str("Enum<");

            let mut first = true;
            for x in &info.variants {
                if first {
                    buf.push_str(&(get_type_name(reg, x)?).to_string());
                    first = false;
                } else {
                    buf.push_str(&format!(", {}", get_type_name(reg, x)?));
                }
            }

            buf.push('>');

            buf
        }
        CoreTypeConcrete::Struct(info) => {
            let mut buf = String::new();

            buf.push('(');

            let mut first = true;
            for x in &info.members {
                if first {
                    buf.push_str(&(get_type_name(reg, x)?).to_string());
                    first = false;
                } else {
                    buf.push_str(&format!(", {}", get_type_name(reg, x)?));
                }
            }

            buf.push(')');

            buf
        }
        CoreTypeConcrete::Felt252Dict(_) => todo!(),
        CoreTypeConcrete::Felt252DictEntry(_) => todo!(),
        CoreTypeConcrete::SquashedFelt252Dict(_) => todo!(),
        CoreTypeConcrete::Pedersen(_) => todo!(),
        CoreTypeConcrete::Poseidon(_) => todo!(),
        CoreTypeConcrete::Span(_) => todo!(),
        CoreTypeConcrete::StarkNet(_) => todo!(),
        CoreTypeConcrete::SegmentArena(_) => todo!(),
        CoreTypeConcrete::Snapshot(_) => todo!(),
        CoreTypeConcrete::Bytes31(_) => todo!(),
        CoreTypeConcrete::BoundedInt(_) => todo!(),
    })
}
