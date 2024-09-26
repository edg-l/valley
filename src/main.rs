use std::{error::Error, path::PathBuf};

use cairo_lang_sierra::{
    extensions::{
        core::{CoreLibfunc, CoreType},
        ConcreteType,
    },
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

        buffunc.push_str(&format!("pub fn f_{}(", func.id.id));

        let mut first = true;
        for (i, param) in func.signature.param_types.iter().enumerate() {
            let t = reg.get_type(param)?;

            if first {
                buffunc.push_str(&format!("p{i}: {}", t.info().long_id.generic_id.0));
                first = false;
            } else {
                buffunc.push_str(&format!(", p{i}: {}", t.info().long_id.generic_id.0));
            }
        }

        buffunc.push_str(") -> (");

        first = true;
        for param in func.signature.ret_types.iter() {
            let t = reg.get_type(param)?;

            if first {
                buffunc.push_str(&format!("{}", t.info().long_id.generic_id.0));
                first = false;
            } else {
                buffunc.push_str(&format!(", {}", t.info().long_id.generic_id.0));
            }
        }

        buffunc.push_str(") {\n");
        buffunc.push_str("}\n");

        buf.push_str(&buffunc);
    }

    std::fs::write("out.cairo_dec", buf)?;
    Ok(())
}
