mod analyse;
mod ast;
mod codegen;
mod compiler;
mod config;
mod execution;
mod lex;
mod parse;
mod r#type;

use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{env, vec};
use std::{fs, io::Write};

use analyse::TypeCheckError;
use codegen::{BytecodeGenerator, Chunk, CodegenContext, CodegenResult, SlotManager};
use compiler::{CompilationResult, Compiler, CompilerFlags};
use config::Config;
use deepsize::DeepSizeOf;
use execution::ExecutionResult;
use execution::Heap;
use execution::ProcessContext;
use execution::{ExecutionEngine, SharedExecutionContext};

fn repl() {
    // let config = Config { max_memory: 1000 };
    // let mut shared_execution_context = SharedExecutionContext {
    //     heap: Heap {
    //         live_slots: vec![],
    //         dead_objects: vec![],
    //     },
    // };
    // let mut codegen_context = CodegenContext {
    //     current_chunk_pointer: 0,
    //     chunks: vec![Chunk {
    //         current_register: 0,
    //         debug_line_info: vec![],
    //         constant_pool: vec![],
    //         gc_ref_data: vec![],
    //         instructions: vec![],
    //         variable_map: HashMap::new(),
    //         string_interns: HashMap::new(),
    //     }],
    // };
    // let mut environment = ProcessContext {
    //     stack_frame_pointer: 0,
    //     stack_frames: vec![],
    //     native_fns: HashMap::new(),
    // };

    // load_prelude(
    //     &config,
    //     &mut shared_execution_context,
    //     &mut codegen_context,
    //     &mut environment,
    // );

    // let mut lexer = lex::Lexer::new();
    // let mut bytecode_generator = BytecodeGenerator::new(&config, &mut codegen_context);
    // let mut exec_engine =
    //     ExecutionEngine::new(&config, &mut shared_execution_context, &mut environment);

    // loop {
    //     let mut line = String::new();
    //     print!(">>");
    //     io::stdout().flush();
    //     std::io::stdin().read_line(&mut line).unwrap();
    //     let tokens = lexer.lex(line);
    //     let mut parser = parse::Parser {
    //         tokens: &tokens,
    //         counter: 0,
    //     };
    //     let ast = parser.parse();
    //     // each time this iterates, it incrementally compiles, is this what we want... probably?
    //     let bytecode = bytecode_generator.generate(&ast);
    //     // println!("generated bytecode {:#?}", bytecode);
    //     let result = exec_engine.exec("anon".to_string(), bytecode, true);
    //     match result {
    //         Ok(o) => {
    //             println!("={:?}", o);
    //         }
    //         Err(e) => {
    //             println!("encountered runtime exception {:?}", e);
    //             exec_engine.print_stacktrace();
    //         }
    //     }
    // }
}

fn print_typecheck_error(source: String, typecheck_err: TypeCheckError) {
    println!("Typecheck Error:\n");
    let split_source = source.lines().collect::<Vec<&str>>();

    match typecheck_err {
        TypeCheckError::TYPE_NOT_ASSIGNABLE(lhs, rhs, lhs_type, rhs_type) => {
            println!("{}", split_source[lhs.line as usize]);
            let left_squiggle = "^".repeat((lhs.index_end - lhs.index) as usize);
            let right_squiggle = "^".repeat((rhs.index_end - rhs.index) as usize);
            let offset = rhs.index - lhs.index + 1;
            println!(
                "{}{}{}{}",
                " ".repeat(lhs.index as usize),
                left_squiggle,
                " ".repeat(offset as usize),
                right_squiggle
            );
            println!("{:?} not assignable to {:?}.\n", rhs_type, lhs_type)
        }
        TypeCheckError::UNKNOWN_VARIABLE(t) => {
            println!("{}", split_source[t.pos.line as usize]);
            let left_squiggle = "^".repeat((t.pos.index_end - t.pos.index) as usize);
            println!("{}{}", " ".repeat(t.pos.index as usize), left_squiggle);
            println!("unknown variable {:?}.\n", t.typ);
        }
        TypeCheckError::UNKNOWN_DATA_TYPE(data_type, pos) => {
            println!("{}", split_source[pos.line as usize]);
            let left_squiggle = "^".repeat((pos.index_end - pos.index) as usize);
            println!("{}{}", " ".repeat(pos.index as usize), left_squiggle);
            println!("unknown data type {:?}.\n", data_type);
        }
        TypeCheckError::MISSING_ARGUMENT => println!("missing argument"),
    }
}

fn exec(file_to_exec: String) {
    let mut compiler = Compiler::new();

    fs::create_dir_all("./gila-build");

    let config = Config {
        max_memory: 100_000,
    };
    let mut shared_execution_context = SharedExecutionContext {
        heap: Heap {
            live_slots: vec![],
            dead_objects: vec![],
        },
        gila_abis_dlls: vec![],
    };

    let mut codegen_context = CodegenContext {
        current_chunk_pointer: 0,
        chunks: vec![Chunk {
            slot_manager: SlotManager::new(),
            debug_line_info: vec![],
            constant_pool: vec![],
            gc_ref_data: vec![],
            instructions: vec![],
            variable_map: HashMap::new(),
            string_interns: HashMap::new(),
        }],
    };
    let mut environment = ProcessContext {
        stack_frame_pointer: 0,
        stack_frames: vec![],
        native_fns: HashMap::new(),
    };

    let prelude_source = fs::read_to_string("./prelude/prelude.gila").expect("Unable to read file");
    let prelude_compile_result = compiler.compile_and_exec(
        "prelude".to_string(),
        CompilerFlags {
            init_builtins: true,
        },
        prelude_source,
        config.clone(),
        Some(codegen_context.clone()),
        Some(environment.clone()),
        Some(shared_execution_context.clone()),
    );

    shared_execution_context = prelude_compile_result
        .execution_result
        .shared_execution_context;
    environment = prelude_compile_result.execution_result.process_context;
    codegen_context = prelude_compile_result.codegen_result.codegen_context;

    let source = fs::read_to_string(file_to_exec.to_string()).expect("Unable to read file");

    let result = compiler.compile_and_exec(
        file_to_exec.to_string(),
        CompilerFlags {
            init_builtins: false,
        },
        source,
        config,
        Some(codegen_context),
        Some(environment),
        Some(shared_execution_context),
    );

    match result.execution_result.result {
        Ok(o) => {}
        Err(e) => {
            println!("encountered runtime exception {:?}", e);
            // execution_engine.print_stacktrace();
        }
    }
    let denominator = 1000_000;
    println!(
        "compiled in {:.9?}s executed in {:.9?}s & used {:.9?}MB",
        result.compilation_time.as_secs_f64(),
        result.execution_time.as_secs_f64(),
        result
            .execution_result
            .shared_execution_context
            .heap
            .live_slots
            .deep_size_of()
            / denominator
    );
}

fn do_test(file_to_test: String) {
    let start = Instant::now();
    println!("testing {}...", file_to_test);

    fs::create_dir_all("./gila-build");

    let mut compiler = Compiler::new();

    let config = Config {
        max_memory: 100_000,
    };
    let mut shared_execution_context = SharedExecutionContext {
        heap: Heap {
            live_slots: vec![],
            dead_objects: vec![],
        },
        gila_abis_dlls: vec![],
    };

    let mut codegen_context = CodegenContext {
        current_chunk_pointer: 0,
        chunks: vec![Chunk {
            slot_manager: SlotManager::new(),
            debug_line_info: vec![],
            constant_pool: vec![],
            gc_ref_data: vec![],
            instructions: vec![],
            variable_map: HashMap::new(),
            string_interns: HashMap::new(),
        }],
    };
    let mut environment = ProcessContext {
        stack_frame_pointer: 0,
        stack_frames: vec![],
        native_fns: HashMap::new(),
    };

    let prelude_source = fs::read_to_string("./prelude/prelude.gila").expect("Unable to read file");
    let prelude_compile_result = compiler.compile_and_exec(
        "prelude".to_string(),
        CompilerFlags {
            init_builtins: true,
        },
        prelude_source,
        config.clone(),
        Some(codegen_context.clone()),
        Some(environment.clone()),
        Some(shared_execution_context.clone()),
    );

    codegen_context = prelude_compile_result.codegen_result.codegen_context;
    environment = prelude_compile_result.execution_result.process_context;
    shared_execution_context = prelude_compile_result
        .execution_result
        .shared_execution_context;

    let source = fs::read_to_string(file_to_test.to_string()).expect("Unable to read file");

    let result = compiler.compile_and_exec(
        file_to_test.to_string(),
        CompilerFlags {
            init_builtins: false,
        },
        source,
        config.clone(),
        Some(codegen_context.clone()),
        Some(environment),
        Some(shared_execution_context),
    );

    codegen_context = result.clone().codegen_result.codegen_context;
    environment = result.clone().execution_result.process_context;
    shared_execution_context = result.clone().execution_result.shared_execution_context;

    let mut tests: Vec<Rc<String>> = vec![];
    // println!("variable map {:?}", codegen_context.chunks[0].variable_map);
    for (var, _) in &codegen_context.chunks[0].variable_map {
        if var.starts_with("test_") {
            println!("collected test {}...", var);
            tests.push(var.clone());
        }
    }

    for test in tests {
        let res = compiler.compile_and_exec(
            test.to_string(),
            CompilerFlags {
                init_builtins: false,
            },
            format!("{}()", test.to_string()),
            config.clone(),
            Some(codegen_context.clone()),
            Some(environment.clone()),
            Some(shared_execution_context.clone()),
        );

        // let res = exec_shared_ctx(
        //     format!("{}()", test.to_string()),
        //     config.clone(),
        //     shared_execution_context.clone(),
        //     codegen_context.clone(),
        //     environment.clone(),
        // );
        shared_execution_context = res.execution_result.shared_execution_context;
        let res_unwrapped = res.execution_result.result;
        match res_unwrapped {
            Ok(obj) => {
                let dynamic = obj.as_dynamic_object(&shared_execution_context);
                match dynamic {
                    Ok(dynamic_obj) => {
                        if dynamic_obj.fields.contains_key("Data") {
                            println!("doing {:<25}... {}.", test, "✅");
                        } else {
                            println!("doing {:<25}... {}.", test, "❌");
                        }
                    }
                    _ => println!("doing {:<25}... {}.", test, "❌"),
                }
            }
            Err(e) => println!("doing {:<25}... {}.", test, "❌"),
        }
    }

    let denominator = 1000_000;
    println!(
        "compiled in {:.9?}s finished in {:.9?}s & used {:.9?}MB",
        result.compilation_time.as_secs_f64(),
        result.execution_time.as_secs_f64(),
        result
            .execution_result
            .shared_execution_context
            .heap
            .live_slots
            .deep_size_of()
            / denominator
    );
}

enum Mode {
    FILE(String),
    REPL,
    TEST(String),
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_to_exec: String = args[3].to_string();
    // let mode = Mode::TEST(file_to_exec);
    let mode = Mode::FILE(file_to_exec);
    // let mode = Mode::REPL;

    match mode {
        Mode::FILE(path) => exec(path),
        Mode::REPL => repl(),
        Mode::TEST(path) => do_test(path),
    }
}
