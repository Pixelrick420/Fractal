use parser::BinOp;
use parser::Expr;
use parser::Program;
use parser::Stmt;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn analyze(parse_result: Result<Program, String>) -> Result<Program, String> {
    let program = parse_result?;
    let mut defined_vars: HashSet<String> = HashSet::new();

    for stmt in &program.statements {
        analyze_statement(stmt, &mut defined_vars)?;
    }

    return Ok(program);
}

fn analyze_statement(stmt: &Stmt, defined_vars: &mut HashSet<String>) -> Result<(), String> {
    match stmt {
        Stmt::Assignment { name, value } => {
            analyze_expression(value, defined_vars)?;
            defined_vars.insert(name.clone());

            Ok(())
        }

        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            analyze_expression(condition, defined_vars)?;

            let mut then_vars = defined_vars.clone();
            for stmt in then_body {
                analyze_statement(stmt, &mut then_vars)?;
            }

            if let Some(else_stmts) = else_body {
                let mut else_vars = defined_vars.clone();
                for stmt in else_stmts {
                    analyze_statement(stmt, &mut else_vars)?;
                }
            }

            Ok(())
        }

        Stmt::For {
            init,
            condition,
            increment,
            body,
        } => {
            analyze_statement(init, defined_vars)?;
            analyze_expression(condition, defined_vars)?;
            let mut loop_vars = defined_vars.clone();
            analyze_statement(increment, &mut loop_vars)?;
            for stmt in body {
                analyze_statement(stmt, &mut loop_vars)?;
            }

            Ok(())
        }

        Stmt::While { condition, body } => {
            analyze_expression(condition, defined_vars)?;
            let mut loop_vars = defined_vars.clone();
            for stmt in body {
                analyze_statement(stmt, &mut loop_vars)?;
            }

            Ok(())
        }

        Stmt::Exit { code } => {
            analyze_expression(code, defined_vars)?;
            Ok(())
        }
    }
}

fn analyze_expression(expr: &Expr, defined_vars: &HashSet<String>) -> Result<(), String> {
    match expr {
        Expr::IntLit(_) => Ok(()),

        Expr::Ident(name) => {
            if defined_vars.contains(name) {
                Ok(())
            } else {
                Err(format!("Error: Use of undefined variable '{}'", name))
            }
        }

        Expr::BinaryOp { left, op, right } => {
            analyze_expression(left, defined_vars)?;
            analyze_expression(right, defined_vars)?;
            Ok(())
        }
    }
}
