use async_trait::async_trait;
use crate::error::ActionResult;
use std::marker::PhantomData;

/// Trait for executing actions with parameters
#[async_trait]
pub trait ActionExecutor {
    /// The input type for this executor
    type Input;
    /// The output type for this executor
    type Output;

    /// Execute the action with the given input
    async fn execute(&self, input: Self::Input) -> ActionResult<Self::Output>;
}

/// A simple executor that stores a function to execute
#[derive(Clone)]
pub struct FunctionExecutor<I, O, F>
where
    F: Fn(I) -> ActionResult<O> + Send + Sync,
{
    function: F,
    _input_type: PhantomData<I>,
    _output_type: PhantomData<O>,
}

impl<I, O, F> FunctionExecutor<I, O, F>
where
    F: Fn(I) -> ActionResult<O> + Send + Sync,
{
    /// Create a new function executor with the given function
    pub fn new(function: F) -> Self {
        Self { 
            function,
            _input_type: PhantomData,
            _output_type: PhantomData,
        }
    }
}

#[async_trait]
impl<I, O, F> ActionExecutor for FunctionExecutor<I, O, F>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
    F: Fn(I) -> ActionResult<O> + Send + Sync + 'static,
{
    type Input = I;
    type Output = O;

    async fn execute(&self, input: Self::Input) -> ActionResult<Self::Output> {
        (self.function)(input)
    }
} 