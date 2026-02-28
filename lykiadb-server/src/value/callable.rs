use super::RV;
use crate::interpreter::{HaltReason, Interpreter, environment::EnvironmentFrame, expr::ProgramState};
use interb::Symbol;
use lykiadb_common::memory::Shared;
use lykiadb_lang::{
    ast::{Span, stmt::Stmt}, types::Datatype
};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

pub trait Aggregator<'v> {
    fn row(&mut self, row: &RV<'v>);
    fn finalize(&self) -> RV<'v>;
}

#[derive(Clone, Debug)]
pub struct RVCallable<'v> {
    pub function: Arc<Function<'v>>,
    pub parameter_types: Datatype,
    pub return_type: Datatype,
}

impl<'v> RVCallable<'v> {
    pub fn new(function: Function<'v>, input_type: Datatype, return_type: Datatype) -> Self {
        RVCallable {
            function: Arc::new(function),
            parameter_types: input_type,
            return_type,
        }
    }

    pub fn is_agg(&self) -> bool {
        matches!(&*self.function, Function::Agg { .. })
    }

    pub fn call(
        &self,
        state: &ProgramState<'v>,
        called_from: &Span,
        arguments: &[RV<'v>],
    ) -> Result<RV<'v>, HaltReason<'v>> {
        let mut interpreter = Interpreter::from_state(state);
        match &self.function.as_ref() {
            Function::Stateful(stateful) => stateful.write().unwrap().call(&mut interpreter, arguments),
            Function::Native { function } => function(&mut interpreter, called_from, arguments),
            Function::Agg { function, .. } => {
                let mut aggregator = function();

                if let RV::Array(arr) = &arguments[0] {
                    for item in arr.iter() {
                        aggregator.row(&item);
                    }
                } else {
                    for item in arguments.iter() {
                        aggregator.row(item);
                    }
                }

                Ok(aggregator.finalize())
            }
            Function::UserDefined {
                parameters,
                closure,
                body,
                ..
            } => interpreter.user_fn_call(body, closure.clone(), parameters, arguments),
        }
    }
}

pub trait Stateful<'v> {
    fn call(
        &mut self,
        interpreter: &mut Interpreter<'v>,
        rv: &[RV<'v>],
    ) -> Result<RV<'v>, HaltReason<'v>>;
}

pub type AggregatorFactory<'v> = fn() -> Box<dyn Aggregator<'v> + Send>;

#[derive(Clone)]
pub enum Function<'v> {
    Native {
        function: fn(
            &mut Interpreter<'v>,
            called_from: &Span,
            &[RV<'v>],
        ) -> Result<RV<'v>, HaltReason<'v>>,
    },
    Stateful(Shared<dyn Stateful<'v> + Send + Sync + 'v>),
    UserDefined {
        name: Symbol,
        parameters: Vec<Symbol>,
        closure: Arc<EnvironmentFrame<'v>>,
        body: Arc<Vec<Stmt>>,
    },
    Agg {
        name: String,
        function: AggregatorFactory<'v>,
    },
}

impl<'v> Function<'v> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Stateful(_) | Function::Native { .. } => write!(f, "<native_fn>"),
            Function::UserDefined { .. } => write!(f, "<user_defined_fn>"),
            Function::Agg { .. } => write!(f, "<agg_fn>"),
        }
    }
}

impl<'v> Debug for Function<'v> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl<'v> Display for Function<'v> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}
