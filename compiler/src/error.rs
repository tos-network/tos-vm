use thiserror::Error;
use tos_ast::Operator;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("expected a function")]
    ExpectedFunction,
    #[error("too many array values: {0}")]
    TooManyArrayValues(usize),
    #[error("too many map values: {0}")]
    TooManyMapValues(usize),
    #[error("duplicated struct {0}")]
    DuplicatedStruct(u16),
    #[error("duplicated enum {0}")]
    DuplicatedEnum(u16),
    #[error("expected a 'break' statement")]
    ExpectedBreak,
    #[error("expected a 'continue' statement")]
    ExpectedContinue,
    #[error("Missing break patch")]
    MissingBreakPatch,
    #[error("Missing continue patch")]
    MissingContinuePatch,
    #[error("memory store is not empty")]
    MemoryStoreNotEmpty,
    #[error("expected a assignment operator, got {0:?}")]
    ExpectedOperatorAssignment(Operator),
    #[error("unexpected operator {0:?}")]
    UnexpectedOperator(Operator),
    #[error("expected a memory store id")]
    ExpectedMemstoreId,
    #[error("expected a variable")]
    ExpectedVariable,
    #[error("expected a primitive type")]
    ExpectedPrimitiveType,
    #[error("expected a value on the stack")]
    ExpectedValueOnStack,
    #[error("less value on the stack than previous")]
    LessValueOnStackThanPrevious,
    #[error("expected a stack scope")]
    ExpectedStackScope,
    #[error("dangling value on the stack")]
    DanglingValueOnStack,
    #[error("too much dangling value on the stack")]
    TooMuchDanglingValueOnStack,
    #[error("expected a memory scope")]
    ExpectedMemoryScope,
    #[error("Hook {0} is already registered")]
    HookAlreadyRegistered(u8),
}