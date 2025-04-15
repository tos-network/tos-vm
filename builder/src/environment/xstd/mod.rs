mod array;
mod optional;
mod string;
mod integer;
mod range;
mod map;
mod bytes;
mod math;

use std::ptr;

use xelis_types::{Primitive, Type};
use xelis_environment::{
    EnvironmentError,
    FnInstance,
    FnParams,
    FnReturnType,
    Context,
};
use super::EnvironmentBuilder;

pub fn register(env: &mut EnvironmentBuilder) {
    array::register(env);
    bytes::register(env);
    optional::register(env);
    string::register(env);
    integer::register(env);
    range::register(env);
    map::register(env);
    math::register(env);

    env.register_native_function("println", None, vec![("value", Type::Any)], println, 1, None);
    env.register_native_function("debug", None, vec![("value", Type::Any)], debug, 1, None);
    env.register_native_function("panic", None, vec![("value", Type::Any)], panic, 1, Some(Type::Any));
    env.register_native_function("assert", None, vec![("value", Type::Bool)], assert, 1, None);
    env.register_native_function("is_same_ptr", None, vec![("left", Type::Any), ("right", Type::Any)], is_same_ptr, 5, Some(Type::Bool));
    env.register_native_function("require", None, vec![("condition", Type::Bool), ("msg", Type::String)], require, 1, None);
    env.register_native_function("clone", Some(Type::T(None)), vec![], clone, 5, Some(Type::T(None)));
}

fn println(_: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let param = &parameters[0];
    println!("{}", param.as_ref()?);

    Ok(None)
}

fn debug(_: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let param = &parameters[0];
    println!("{:?}", param);

    Ok(None)
}

fn panic(_: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let param = parameters.remove(0);
    let value = param.into_owned()?;

    Err(EnvironmentError::Panic(format!("{:#}", value)))
}

fn assert(_: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let param = &parameters[0];
    let value = param.as_bool()?;

    if value {
        Ok(None)
    } else {
        Err(EnvironmentError::AssertionFailed)
    }
}

fn is_same_ptr(_: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let left = parameters[0].as_ref()?;
    let right = parameters[1].as_ref()?;
    let same = ptr::from_ref(left) == ptr::from_ref(right);

    Ok(Some(Primitive::Boolean(same).into()))
}

fn require(_: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let msg = parameters.remove(1)
        .into_owned()?
        .into_string()?;

    if !msg.chars().all(|c| c.is_alphanumeric() || c == ' ') {
        return Err(EnvironmentError::InvalidExpect);
    }

    let param = &parameters[0];
    let value = param.as_bool()?;

    if value {
        Ok(None)
    } else {
        Err(EnvironmentError::Expect(msg))
    }
}

fn clone(zelf: FnInstance, _: FnParams, context: &mut Context) -> FnReturnType {
    let zelf = zelf?;

    let memory = zelf.calculate_memory_usage(context.memory_left())?;
    // Double cost: computation for cloning and memory allocation?
    // context.increase_gas_usage(memory as _)?;
    context.increase_memory_usage_unchecked(memory)?;

    Ok(Some(zelf.clone()))
}