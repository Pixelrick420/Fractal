use crate::parser::{BinOp, Expr, Program, Stmt};
use std::collections::HashMap;

fn gen_expr(expr: &Expr, output: &mut String, var_offset: &HashMap<String, i32>) {
    match expr {
        Expr::IntLit(val) => {
            output.push_str(&format!("    mov rax, {}\n", val));
        }

        Expr::Ident(name) => {
            let offset = var_offset
                .get(name)
                .expect(&format!("Variable {} not found", name));
            output.push_str(&format!("    mov rax, [rbp-{}]\n", offset));
        }

        Expr::BinaryOp { left, op, right } => match op {
            BinOp::Add => {
                gen_expr(left, output, var_offset);
                output.push_str("    push rax\n");
                gen_expr(right, output, var_offset);
                output.push_str("    mov rbx, rax\n");
                output.push_str("    pop rax\n");
                output.push_str("    add rax, rbx\n");
            }

            BinOp::Sub => {
                gen_expr(left, output, var_offset);
                output.push_str("    push rax\n");
                gen_expr(right, output, var_offset);
                output.push_str("    mov rbx, rax\n");
                output.push_str("    pop rax\n");
                output.push_str("    sub rax, rbx\n");
            }

            BinOp::Mul => {
                gen_expr(left, output, var_offset);
                output.push_str("    push rax\n");
                gen_expr(right, output, var_offset);
                output.push_str("    mov rbx, rax\n");
                output.push_str("    pop rax\n");
                output.push_str("    imul rax, rbx\n");
            }

            BinOp::Greater => {
                gen_expr(left, output, var_offset);
                output.push_str("    push rax\n");
                gen_expr(right, output, var_offset);
                output.push_str("    mov rbx, rax\n");
                output.push_str("    pop rax\n");
                output.push_str("    cmp rax, rbx\n");
                output.push_str("    setg al\n");
                output.push_str("    movzx rax, al\n");
            }

            BinOp::Less => {
                gen_expr(left, output, var_offset);
                output.push_str("    push rax\n");
                gen_expr(right, output, var_offset);
                output.push_str("    mov rbx, rax\n");
                output.push_str("    pop rax\n");
                output.push_str("    cmp rax, rbx\n");
                output.push_str("    setl al\n");
                output.push_str("    movzx rax, al\n");
            }

            BinOp::Assign => {
                panic!("Assignment in expression context");
            }
        },
    }
}

fn allocate_var(name: &str, var_offset: &mut HashMap<String, i32>, stack_pos: &mut i32) {
    if !var_offset.contains_key(name) {
        *stack_pos += 8;
        var_offset.insert(name.to_string(), *stack_pos);
    }
}

fn gen_stmt(
    stmt: &Stmt,
    output: &mut String,
    var_offset: &mut HashMap<String, i32>,
    stack_pos: &mut i32,
    label_counter: &mut usize,
) {
    match stmt {
        Stmt::Assignment { name, value } => {
            allocate_var(name, var_offset, stack_pos);
            gen_expr(value, output, var_offset);
            let offset = var_offset.get(name).unwrap();
            output.push_str(&format!("    mov [rbp-{}], rax\n", offset));
        }

        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let else_label = format!("else_{}", label_counter);
            let end_label = format!("end_if_{}", label_counter);
            *label_counter += 1;

            gen_expr(condition, output, var_offset);
            output.push_str("    cmp rax, 0\n");
            output.push_str(&format!("    je {}\n", else_label));

            for stmt in then_body {
                gen_stmt(stmt, output, var_offset, stack_pos, label_counter);
            }
            output.push_str(&format!("    jmp {}\n", end_label));

            output.push_str(&format!("{}:\n", else_label));
            if let Some(else_stmts) = else_body {
                for stmt in else_stmts {
                    gen_stmt(stmt, output, var_offset, stack_pos, label_counter);
                }
            }

            output.push_str(&format!("{}:\n", end_label));
        }

        Stmt::While { condition, body } => {
            let start_label = format!("while_start_{}", label_counter);
            let end_label = format!("while_end_{}", label_counter);
            *label_counter += 1;

            output.push_str(&format!("{}:\n", start_label));

            gen_expr(condition, output, var_offset);
            output.push_str("    cmp rax, 0\n");
            output.push_str(&format!("    je {}\n", end_label));

            for stmt in body {
                gen_stmt(stmt, output, var_offset, stack_pos, label_counter);
            }

            output.push_str(&format!("    jmp {}\n", start_label));
            output.push_str(&format!("{}:\n", end_label));
        }

        Stmt::For {
            init,
            condition,
            increment,
            body,
        } => {
            let start_label = format!("for_start_{}", label_counter);
            let end_label = format!("for_end_{}", label_counter);
            *label_counter += 1;

            gen_stmt(init, output, var_offset, stack_pos, label_counter);

            output.push_str(&format!("{}:\n", start_label));

            gen_expr(condition, output, var_offset);
            output.push_str("    cmp rax, 0\n");
            output.push_str(&format!("    je {}\n", end_label));

            for stmt in body {
                gen_stmt(stmt, output, var_offset, stack_pos, label_counter);
            }

            gen_stmt(increment, output, var_offset, stack_pos, label_counter);

            output.push_str(&format!("    jmp {}\n", start_label));
            output.push_str(&format!("{}:\n", end_label));
        }

        Stmt::Exit { code } => {
            gen_expr(code, output, var_offset);
            output.push_str("    mov rdi, rax\n");
            output.push_str("    mov rax, 60\n");
            output.push_str("    syscall\n");
        }
    }
}

pub fn generate(program: &Program) -> String {
    let mut output = String::new();
    let mut var_offset: HashMap<String, i32> = HashMap::new();
    let mut stack_pos: i32 = 0;
    let mut label_counter: usize = 0;

    output.push_str("BITS 64\n");
    output.push_str("section .text\n");
    output.push_str("global _start\n");
    output.push_str("\n");
    output.push_str("_start:\n");

    output.push_str("    push rbp\n");
    output.push_str("    mov rbp, rsp\n");
    output.push_str("    sub rsp, 1024\n");

    for stmt in &program.statements {
        gen_stmt(
            stmt,
            &mut output,
            &mut var_offset,
            &mut stack_pos,
            &mut label_counter,
        );
    }

    output.push_str("\n");
    output.push_str("    ; Default exit with code 0\n");
    output.push_str("    mov rax, 60\n");
    output.push_str("    xor rdi, rdi\n");
    output.push_str("    syscall\n");

    output
}
