use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Parser, Debug)]
struct Args {
    /// input compilation database
    #[arg(short, long, default_value = "compile_commands.json")]
    compile_commands: String,

    /// file where the modified compilation database will be written
    #[arg(short, long, default_value = "compile_commands.json")]
    output: String,

    /// adds the specified include directory to all compilation units in the database
    #[arg(short = 'i', long)]
    add_include: Vec<String>,

    /// removes the specified include directory from all compilation units in the database
    #[arg(short = 'd', long)]
    delete_include: Vec<String>,

    /// adds the specified compile arguments to all compilation units in the database
    #[arg(long)]
    add_arg: Vec<String>,

    /// removes the specified compile arguments from all compilation units in the database
    #[arg(long)]
    delete_arg: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.add_include.is_empty()
        && args.delete_include.is_empty()
        && args.add_arg.is_empty()
        && args.delete_arg.is_empty()
    {
        println!("No modifications requested, exiting.");
        return Ok(());
    }

    // load compile commands file into memory
    let compile_commands_reader = BufReader::new(
        File::open(&args.compile_commands)
            .with_context(|| format!("Could not open {}", args.compile_commands))?,
    );

    // parse the json and ensure that the structure is always an array of objects
    let compile_commands = serde_json::from_reader(compile_commands_reader)
        .with_context(|| format!("Could not parse {} as json", args.compile_commands))?;
    let mut compile_commands: Vec<Value> = match compile_commands {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![compile_commands],
        _ => {
            return Err(anyhow!(
            "{} file was not formatted correctly, the top level item must be an array or object",
            args.compile_commands
        ))
        }
    };

    for compile_unit in compile_commands.iter_mut() {
        // a compile command file consists of "compile units" each of which is a json object,
        // here we unpack that from the json structure
        let Value::Object(map) = compile_unit else {
            return Err(anyhow!("{} file was not formatted correctly. The following compile unit was not in the form of a JSON object: {}", args.compile_commands, compile_unit));
        };

        // copy the filename out of the json object (for use in error messages)
        let Value::String(name) = map
            .get("file")
            .with_context(|| {
                format!(
                    "The following compile unit did not have a command field: {:?}",
                    map
                )
            })?
            .clone()
        else {
            return Err(anyhow!(
                "the following compile unit's file name was not a string: {:?}",
                map
            ));
        };

        // get a mutable reference to the compile command used for this compile unit
        let Value::String(compile_command) = map.get_mut("command").with_context(|| {
            format!(
                "The following compile unit did not have a command field: {}",
                name
            )
        })?
        else {
            return Err(anyhow!(
                "the following compile unit's command field was not a string: {}",
                name
            ));
        };

        // modify the compile command as specified by the command line arguments (e.g add & remove include dirs)
        for include in &args.add_include {
            if let Some(index) = compile_command.find("-I") {
                compile_command.insert_str(index, &format!(" -I{} ", include));
            }
        }

        for include in &args.delete_include {
            let target_str = format!(" -I{}", include);
            if let Some(start_idx) = compile_command.find(&target_str) {
                compile_command.replace_range(start_idx..start_idx + target_str.len(), "");
            }
        }

        for arg in &args.add_arg {
            compile_command.push_str(&format!(" -{}", arg));
        }

        for arg in &args.delete_arg {
            let target_str = format!(" -{}", arg);
            if let Some(start_idx) = compile_command.find(&target_str) {
                compile_command.replace_range(start_idx..start_idx + target_str.len(), "");
            }
        }
    }

    // write modified compile commands back out
    let compile_commands_writer = BufWriter::new(
        File::create(&args.output).with_context(|| format!("Could not open {}", args.output))?,
    );
    serde_json::to_writer_pretty(compile_commands_writer, &Value::Array(compile_commands))
        .with_context(|| "Could not serialize modified compile commands as JSON")?;

    Ok(())
}
