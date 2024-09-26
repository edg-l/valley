use std::{error::Error, path::PathBuf};

use cairo_lang_sierra::{
    extensions::{
        core::{CoreLibfunc, CoreType, CoreTypeConcrete},
        ConcreteType,
    },
    ids::ConcreteTypeId,
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

    for func in &program.funcs {
        let mut buffunc = String::new();

        buffunc.push_str(&format!("pub fn func_{}(", func.id.id));

        let mut first = true;
        for (i, param) in func.signature.param_types.iter().enumerate() {
            let ty_name = get_type_name(&reg, param)?;

            if first {
                buffunc.push_str(&format!("p{i}: {ty_name}"));
                first = false;
            } else {
                buffunc.push_str(&format!(", p{i}: {ty_name}"));
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
        buffunc.push_str("}\n");

        buf.push_str(&buffunc);
    }

    std::fs::write("out.cairo_dec", buf)?;
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
        CoreTypeConcrete::Const(_) => todo!(),
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
