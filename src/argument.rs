use nu_protocol::{ast::Expression, Span, Spanned};

use crate::{into_expression::IntoExpression, NewEmpty};

/// A struct representing the argument to a function
pub enum Argument {
    Named((String, Option<Expression>)),
    Positional(Expression),
}

impl Argument {
    /// Creates a new named argument. No value means passing the argument as a flag (like --verbose)
    #[inline]
    pub fn named<S: ToString, E: IntoExpression>(name: S, value: Option<E>) -> Self {
        Self::Named((name.to_string(), value.map(|v| v.into_expression())))
    }

    /// Creates a new positional argument
    #[inline]
    pub fn positional<E: IntoExpression>(value: E) -> Self {
        Self::Positional(value.into_expression())
    }

    pub(crate) fn into_nu_argument(self) -> nu_protocol::ast::Argument {
        match self {
            Argument::Named((name, value)) => nu_protocol::ast::Argument::Named((
                Spanned {
                    item: name,
                    span: Span::empty(),
                },
                None,
                value,
            )),
            Argument::Positional(value) => nu_protocol::ast::Argument::Positional(value),
        }
    }
}

/// Converts a given type into an argument
pub trait IntoArgument {
    fn into_argument(self) -> Argument;
}

impl<E: IntoExpression> IntoArgument for E {
    #[inline]
    fn into_argument(self) -> Argument {
        Argument::positional(self)
    }
}

impl IntoArgument for Argument {
    #[inline]
    fn into_argument(self) -> Argument {
        self
    }
}
