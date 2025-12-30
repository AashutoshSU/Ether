// use inkwell::builder::Builder;
// use inkwell::context::Context;
// use inkwell::module::Module;
// use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
// use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
// use std::collections::HashMap;
//
// pub struct CodeGen<'ctx> {
//     context: &'ctx Context,
//     module: Module<'ctx>,
//     builder: Builder<'ctx>,
//     // Symbol tables
//     variables: HashMap<String, PointerValue<'ctx>>,
//     functions: HashMap<String, FunctionValue<'ctx>>,
//     structs: HashMap<String, BasicTypeEnum<'ctx>>,
//     // Current function being compiled
//     current_function: Option<FunctionValue<'ctx>>,
// }
//
// impl<'ctx> CodeGen<'ctx> {
//     fn ast_type_to_llvm(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>> {
//         match ty {
//             Type::Primitive(name) => match name.as_str() {
//                 "int" => Ok(self.context.i64_type().into()),
//                 "float" => Ok(self.context.f64_type().into()),
//                 "bool" => Ok(self.context.bool_type().into()),
//                 "char" => Ok(self.context.i8_type().into()),
//                 "string" => Ok(self
//                     .context
//                     .i8_type()
//                     .ptr_type(AddressSpace::default())
//                     .into()),
//                 _ => Err(CodeGenError::UnknownType(name.clone())),
//             },
//             Type::Array(inner) => {
//                 let inner_ty = self.ast_type_to_llvm(inner)?;
//                 Ok(inner_ty.ptr_type(AddressSpace::default()).into())
//             }
//             Type::Custom(name) => self
//                 .structs
//                 .get(name)
//                 .cloned()
//                 .ok_or_else(|| CodeGenError::UndefinedStruct(name.clone())),
//             Type::Function(params, ret) => {
//                 // Function types are handled specially
//                 unimplemented!("Function types as values")
//             }
//         }
//     }
// }
//
// fn generate_struct(&mut self, struct_def: &StructDef) -> Result<()> {
//     let field_types: Vec<BasicTypeEnum> = struct_def
//         .fields
//         .iter()
//         .map(|(_, ty)| self.ast_type_to_llvm(ty))
//         .collect::<Result<_>>()?;
//
//     let struct_type = self.context.struct_type(&field_types, false);
//     self.structs
//         .insert(struct_def.name.clone(), struct_type.into());
//     Ok(())
// }
//
// fn generate_function_declaration(&mut self, func: &Function) -> Result<FunctionValue<'ctx>> {
//     let param_types: Vec<BasicMetadataTypeEnum> = func
//         .params
//         .iter()
//         .map(|(_, ty)| self.ast_type_to_llvm(ty).map(Into::into))
//         .collect::<Result<_>>()?;
//
//     let return_type = self.ast_type_to_llvm(&func.return_type)?;
//
//     let fn_type = match return_type {
//         BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
//         BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
//         // ... handle other types
//         _ => return Err(CodeGenError::InvalidReturnType),
//     };
//
//     let fn_val = self.module.add_function(&func.name, fn_type, None);
//     self.functions.insert(func.name.clone(), fn_val);
//     Ok(fn_val)
// }
//
// fn generate_function_body(&mut self, func: &Function, fn_val: FunctionValue<'ctx>) -> Result<()> {
//     let entry = self.context.append_basic_block(fn_val, "entry");
//     self.builder.position_at_end(entry);
//     self.current_function = Some(fn_val);
//
//     // Allocate parameters
//     for (i, (param_name, param_ty)) in func.params.iter().enumerate() {
//         let param_val = fn_val.get_nth_param(i as u32).unwrap();
//         let alloca = self.create_entry_block_alloca(&param_name, param_ty)?;
//         self.builder.build_store(alloca, param_val);
//         self.variables.insert(param_name.clone(), alloca);
//     }
//
//     // Generate body
//     self.generate_block(&func.body)?;
//
//     // Add return if missing
//     if self
//         .builder
//         .get_insert_block()
//         .unwrap()
//         .get_terminator()
//         .is_none()
//     {
//         if func.return_type == Type::Primitive("void".into()) {
//             self.builder.build_return(None);
//         }
//     }
//
//     Ok(())
// }
//
// fn generate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
//     match stmt {
//         Stmt::Var(var_decl) => {
//             let value = self.generate_expr(&var_decl.value)?;
//             let ty = var_decl.ty.as_ref().unwrap_or(&infer_type(&var_decl.value));
//             let alloca = self.create_entry_block_alloca(&var_decl.name, ty)?;
//             self.builder.build_store(alloca, value);
//             self.variables.insert(var_decl.name.clone(), alloca);
//         }
//         Stmt::Return(expr) => {
//             if let Some(e) = expr {
//                 let val = self.generate_expr(e)?;
//                 self.builder.build_return(Some(&val));
//             } else {
//                 self.builder.build_return(None);
//             }
//         }
//         Stmt::Expr(e) => {
//             self.generate_expr(e)?;
//         }
//         Stmt::If {
//             cond,
//             then_block,
//             else_block,
//         } => {
//             self.generate_if(cond, then_block, else_block.as_ref())?;
//         }
//         Stmt::While { cond, body } => {
//             self.generate_while(cond, body)?;
//         }
//         Stmt::For { name, iter, body } => {
//             self.generate_for(name, iter, body)?;
//         }
//         Stmt::Block(block) => {
//             self.generate_block(block)?;
//         }
//     }
//     Ok(())
// }
//
// fn generate_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>> {
//     match expr {
//         Expr::Literal(lit) => self.generate_literal(lit),
//         Expr::Identifier(name) => {
//             let ptr = self
//                 .variables
//                 .get(name)
//                 .ok_or_else(|| CodeGenError::UndefinedVariable(name.clone()))?;
//             Ok(self.builder.build_load(*ptr, name))
//         }
//         Expr::Binary(left, op, right) => {
//             let l = self.generate_expr(left)?;
//             let r = self.generate_expr(right)?;
//             self.generate_binary_op(l, op, r)
//         }
//         Expr::Unary(op, operand) => {
//             let val = self.generate_expr(operand)?;
//             self.generate_unary_op(op, val)
//         }
//         Expr::Call(callee, args) => self.generate_call(callee, args),
//         Expr::Assign(lhs, rhs) => {
//             let value = self.generate_expr(rhs)?;
//             let ptr = self.get_lvalue_ptr(lhs)?;
//             self.builder.build_store(ptr, value);
//             Ok(value)
//         }
//         Expr::Field(obj, field) => self.generate_field_access(obj, field),
//         Expr::Index(arr, idx) => self.generate_index(arr, idx),
//     }
// }
//
// fn generate_binary_op(
//     &mut self,
//     left: BasicValueEnum<'ctx>,
//     op: &BinOp,
//     right: BasicValueEnum<'ctx>,
// ) -> Result<BasicValueEnum<'ctx>> {
//     match op {
//         BinOp::Add => Ok(self
//             .builder
//             .build_int_add(left.into_int_value(), right.into_int_value(), "add")
//             .into()),
//         BinOp::Sub => Ok(self
//             .builder
//             .build_int_sub(left.into_int_value(), right.into_int_value(), "sub")
//             .into()),
//         BinOp::Mul => Ok(self
//             .builder
//             .build_int_mul(left.into_int_value(), right.into_int_value(), "mul")
//             .into()),
//         BinOp::Div => Ok(self
//             .builder
//             .build_int_signed_div(left.into_int_value(), right.into_int_value(), "div")
//             .into()),
//         BinOp::Eq => Ok(self
//             .builder
//             .build_int_compare(
//                 IntPredicate::EQ,
//                 left.into_int_value(),
//                 right.into_int_value(),
//                 "eq",
//             )
//             .into()),
//         // ... other operators
//     }
// }
//
// fn generate_if(
//     &mut self,
//     cond: &Expr,
//     then_block: &Block,
//     else_block: Option<&Block>,
// ) -> Result<()> {
//     let cond_val = self.generate_expr(cond)?.into_int_value();
//
//     let then_bb = self
//         .context
//         .append_basic_block(self.current_function.unwrap(), "then");
//     let else_bb = self
//         .context
//         .append_basic_block(self.current_function.unwrap(), "else");
//     let merge_bb = self
//         .context
//         .append_basic_block(self.current_function.unwrap(), "merge");
//
//     self.builder
//         .build_conditional_branch(cond_val, then_bb, else_bb);
//
//     // Generate then block
//     self.builder.position_at_end(then_bb);
//     self.generate_block(then_block)?;
//     if self
//         .builder
//         .get_insert_block()
//         .unwrap()
//         .get_terminator()
//         .is_none()
//     {
//         self.builder.build_unconditional_branch(merge_bb);
//     }
//
//     // Generate else block
//     self.builder.position_at_end(else_bb);
//     if let Some(else_b) = else_block {
//         self.generate_block(else_b)?;
//     }
//     if self
//         .builder
//         .get_insert_block()
//         .unwrap()
//         .get_terminator()
//         .is_none()
//     {
//         self.builder.build_unconditional_branch(merge_bb);
//     }
//
//     self.builder.position_at_end(merge_bb);
//     Ok(())
// }
//
// pub fn compile_program(program: &Program) -> Result<String> {
//     let context = Context::create();
//     let module = context.create_module("ether_module");
//     let builder = context.create_builder();
//
//     let mut codegen = CodeGen::new(&context, module, builder);
//
//     // First pass: declare all structs and functions
//     for decl in &program.declarations {
//         match decl {
//             Declaration::Struct(s) => codegen.generate_struct(s)?,
//             Declaration::Function(f) => {
//                 codegen.generate_function_declaration(f)?;
//             }
//             _ => {}
//         }
//     }
//
//     // Second pass: generate function bodies
//     for decl in &program.declarations {
//         if let Declaration::Function(f) = decl {
//             let fn_val = codegen.functions[&f.name];
//             codegen.generate_function_body(f, fn_val)?;
//         }
//     }
//
//     // Return LLVM IR as string
//     Ok(codegen.module.print_to_string().to_string())
// }
