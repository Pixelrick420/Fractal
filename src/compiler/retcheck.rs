use crate::compiler::parser::ParseNode;

pub fn block_always_returns(stmts: &[ParseNode]) -> bool {
    for stmt in stmts {
        if stmt_always_returns(stmt) {
            return true;
        }
    }
    false
}

fn block_contains_exit_no_break(stmts: &[ParseNode]) -> bool {
    for stmt in stmts {
        if node_contains_exit_no_break(stmt) {
            return true;
        }
    }
    false
}

fn node_contains_exit_no_break(node: &ParseNode) -> bool {
    match node {
        ParseNode::Return { .. } | ParseNode::Exit { .. } => true,
        ParseNode::Break { .. } => false,
        ParseNode::If {
            then_block,
            else_block,
            ..
        } => {
            block_contains_exit_no_break(then_block)
                || else_block
                    .as_deref()
                    .map_or(false, block_contains_exit_no_break)
        }

        ParseNode::For { body, .. } | ParseNode::While { body, .. } => {
            block_contains_exit_no_break(body)
        }
        _ => false,
    }
}

fn stmt_always_returns(node: &ParseNode) -> bool {
    match node {
        ParseNode::Return { .. } | ParseNode::Exit { .. } => true,

        ParseNode::If {
            then_block,
            else_block,
            ..
        } => {
            let then_returns = block_always_returns(then_block);
            let else_returns = match else_block {
                Some(eb) => block_always_returns(eb),
                None => false,
            };
            then_returns && else_returns
        }

        ParseNode::While {
            condition, body, ..
        } => {
            matches!(condition.as_ref(), ParseNode::BoolLit(true))
                && block_contains_exit_no_break(body)
        }

        ParseNode::For { .. } => false,

        _ => false,
    }
}

pub fn check_function_returns(
    func_name: &str,
    return_type: &ParseNode,
    body: &[ParseNode],
    errors: &mut Vec<String>,
) {
    if matches!(return_type, ParseNode::TypeVoid) {
        return;
    }

    if !block_always_returns(body) {
        errors.push(format!(
            "function `{}` does not return a value on all code paths\n   \
             note: every branch of a non-`:void` function must end with `!return <expr>;`\n   \
             note: a bare `!if` without a matching `!else` is not sufficient — \
             add an `!else` branch that also returns, or add a `!return` after the `!if`",
            func_name
        ));
    }
}
