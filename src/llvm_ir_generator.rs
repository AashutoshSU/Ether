use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
use std::collections::HashMap;

use crate::parser::{
    BinOp, Block, Declaration, Expr, Function, FunctionHeader, Import, Literal, Program, Stmt,
    StructDef, Type, UnOp, VarDecl,
};

// Example AST nodes
// enum Expr {
//     Number(i64),
//     Add(Box<Expr>, Box<Expr>),
//     Multiply(Box<Expr>, Box<Expr>),
// }
//
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,

    // Symbol tables
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    structs: HashMap<String, BasicTypeEnum<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        CodeGen {
            context,
            builder,
            module,
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    // Main Entry Point
    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        // First pass: declare all structs
        for decl in &program.declarations {
            if let Declaration::Struct(s) = decl {
                self.declare_struct(s)?;
            }
        }

        // Second pass: declare all functions
        for decl in &program.declarations {
            if let Declaration::Function(f) = decl {
                self.declare_function(f)?;
            }
        }

        // Third pass: compile function bodies and global variables
        for decl in &program.declarations {
            match decl {
                Declaration::Function(f) => self.compile_function(f)?,
                Declaration::Var(v) => self.compile_global_var(v)?,
                Declaration::Struct(_) => {} // Already handled
            }
        }

        Ok(())
    }

    // Type Conversion
    fn convert_type(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            Type::Primitive(name) => match name.as_str() {
                "int" => Ok(self.context.i64_type().into()),
                "float" => Ok(self.context.f64_type().into()),
                "bool" => Ok(self.context.bool_type().into()),
                "char" => Ok(self.context.i8_type().into()),
                "string" => Ok(self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()),
                "void" => Err("void is not a basic type".to_string()),
                _ => Err(format!("Unknown primitive type: {}", name)),
            },
            Type::Array(inner) => {
                let inner_ty = self.convert_type(inner)?;
                Ok(inner_ty.ptr_type(AddressSpace::default()).into())
            }
            Type::Custom(name) => self
                .structs
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Unknown custom type: {}", name)),
            Type::Function(_) => {
                // Function pointers
                Ok(self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into())
            }
        }
    }

    fn is_void_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Primitive(name) if name == "void")
    }
    // pub fn compile_expr(&self, expr: &Expr) -> IntValue<'ctx> {
    //     match expr {
    //         Expr::Number(n) => self.context.i64_type().const_int(*n as u64, false),
    //         Expr::Add(left, right) => {
    //             let lhs = self.compile_expr(left);
    //             let rhs = self.compile_expr(right);
    //             self.builder
    //                 .build_int_add(lhs, rhs, "addtmp")
    //                 .expect("Failed to build add")
    //         }
    //         Expr::Multiply(left, right) => {
    //             let lhs = self.compile_expr(left);
    //             let rhs = self.compile_expr(right);
    //             self.builder
    //                 .build_int_mul(lhs, rhs, "multmp")
    //                 .expect("Failed to build mul")
    //         }
    //     }
    // }

    // pub fn compile(&self) {
    //     // Create a function: i64 main()
    //     let i64_type = self.context.i64_type();
    //     let fn_type = i64_type.fn_type(&[], false);
    //     let function = self.module.add_function("main", fn_type, None);
    //     let basic_block = self.context.append_basic_block(function, "entry");
    //
    //     self.builder.position_at_end(basic_block);
    //
    //     // Example: compile (2 + 3) * 4
    //     let ast = Expr::Multiply(
    //         Box::new(Expr::Add(
    //             Box::new(Expr::Number(2)),
    //             Box::new(Expr::Number(3)),
    //         )),
    //         Box::new(Expr::Number(4)),
    //     );
    //
    //     let result = self.compile_expr(&ast);
    //     self.builder
    //         .build_return(Some(&result))
    //         .expect("Failed to build return");
    // }
    //
    pub fn print_ir(&self) {
        self.module.print_to_stderr();
    }
}

// fn main() {
//     let context = Context::create();
//     let compiler = Compiler::new(&context);
//
//     compiler.compile();
//     compiler.print_ir();
// }
