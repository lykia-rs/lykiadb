use super::RV;
use super::environment::EnvironmentFrame;
use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    exec::aggregation::Aggregator,
    util::Shared,
};
use interb::Symbol;
use lykiadb_lang::{
    ast::{Span, stmt::Stmt},
    types::Datatype,
};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RVCallable<'session> {
    pub function: Arc<Function<'session>>,
    pub parameter_types: Datatype,
    pub return_type: Datatype,
}

impl<'session> RVCallable<'session> {
    pub fn new(function: Function<'session>, input_type: Datatype, return_type: Datatype) -> Self {
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
        interpreter: &'session mut Interpreter<'session>,
        called_from: &Span,
        arguments: &[RV<'session>],
    ) -> Result<RV<'session>, HaltReason<'session>> {
        match &self.function.as_ref() {
            Function::Stateful(stateful) => stateful.write().unwrap().call(interpreter, arguments),
            Function::Native { function } => function(interpreter, called_from, arguments),
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

                let finalized = aggregator.finalize();

                Ok(finalized)
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

pub trait Stateful {
    fn call<'session>(&mut self, interpreter: &mut Interpreter<'session>, rv: &[RV<'session>]) -> Result<RV<'session>, HaltReason<'session>>;
}

pub type AggregatorFactory<'arena> = fn() -> Box<dyn Aggregator<'arena> + Send>;

#[derive(Clone)]
pub enum Function<'arena> {
    Native {
        function: fn(&mut Interpreter, called_from: &Span, &[RV<'arena>]) -> Result<RV<'arena>, HaltReason<'arena>>,
    },
    Stateful(Shared<dyn Stateful + Send + Sync + 'arena>),
    UserDefined {
        name: Symbol,
        parameters: Vec<Symbol>,
        closure: Arc<EnvironmentFrame<'arena>>,
        body: Arc<Vec<Stmt>>,
    },
    Agg {
        name: String,
        function: AggregatorFactory<'arena>,
    },
}

impl<'arena> Function<'arena> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Stateful(_) | Function::Native { .. } => write!(f, "<native_fn>"),
            Function::UserDefined { .. } => write!(f, "<user_defined_fn>"),
            Function::Agg { .. } => write!(f, "<agg_fn>"),
        }
    }
}

impl<'arena> Debug for Function<'arena> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl<'arena> Display for Function<'arena> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}
