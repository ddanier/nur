use nu_protocol::{
    ast::{Expr, Expression},
    Span, Value,
};

use crate::{IntoValue, NewEmpty};

pub trait IntoExpression {
    fn into_expression(self) -> Expression;
}

pub trait ValueIntoExpression {
    fn into_expression(self) -> Expression;
    fn into_expr(self) -> Expr;
}

impl<V: IntoValue> IntoExpression for V {
    #[inline]
    fn into_expression(self) -> Expression {
        self.into_value().into_expression()
    }
}

impl ValueIntoExpression for Value {
    fn into_expression(self) -> Expression {
        let ty = self.get_type();

        Expression {
            expr: self.into_expr(),
            span: Span::empty(),
            ty,
            custom_completion: None,
        }
    }

    fn into_expr(self) -> Expr {
        match self {
            Value::Bool { val, .. } => Expr::Bool(val),
            Value::Int { val, .. } => Expr::Int(val),
            Value::Float { val, .. } => Expr::Float(val),
            Value::Filesize { val, .. } => Expr::Int(val),
            Value::Duration { val, .. } => Expr::Int(val),
            Value::Date { val, .. } => Expr::DateTime(val),
            Value::String { val, .. } => Expr::String(val),
            Value::Record {
                mut cols, mut vals, ..
            } => {
                let mut entries = Vec::new();

                for _ in 0..cols.len() {
                    let col = cols.remove(0).into_expression();
                    let val = vals.remove(0).into_expression();
                    entries.push((col, val));
                }

                Expr::Record(entries)
            }
            Value::List { vals, .. } => {
                let vals = vals.into_iter().map(|v| v.into_expression()).collect();
                Expr::List(vals)
            }
            Value::Block { val, .. } => Expr::Block(val),
            Value::Nothing { .. } => Expr::Nothing,
            Value::Error { error } => Expr::String(error.to_string()),
            Value::Binary { val, .. } => Expr::Binary(val),
            Value::CellPath { val, .. } => Expr::CellPath(val),
            _ => Expr::Nothing,
        }
    }
}
