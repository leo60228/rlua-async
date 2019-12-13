use futures::prelude::*;
use futures::task::{Context, Poll};
use futures_timer::Delay;
use rlua::{Error, Lua, Result, Thread, UserData, UserDataMethods, Value};
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AsyncCall {
    Sleep(u64),
    Yield,
}

impl UserData for AsyncCall {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct AsyncApi;

impl UserData for AsyncApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("yield", |_, _, _: ()| Ok(AsyncCall::Yield));

        methods.add_method("sleep", |_, _, duration: u64| {
            Ok(AsyncCall::Sleep(duration))
        });
    }
}

pub async fn exec_async(code: impl AsRef<[u8]>) -> Result<()> {
    let lua = Lua::new();

    let (ret, thread) = lua.context(|lua_ctx| {
        lua_ctx
            .load(include_str!("api.lua"))
            .set_name("async")?
            .call(AsyncApi)?;
        let code = lua_ctx.load(&code).set_name("code")?.into_function()?;

        let thread = lua_ctx.create_thread(code)?;

        Ok((
            lua_ctx.create_registry_value(Value::Nil)?,
            lua_ctx.create_registry_value(thread)?,
        ))
    })?;

    loop {
        let val: Result<AsyncCall> = lua.context(|lua_ctx| {
            let thread: Thread = lua_ctx.registry_value(&thread)?;
            let ret: Value = lua_ctx.registry_value(&ret)?;

            thread.resume(ret)
        });

        match val {
            Ok(AsyncCall::Yield) => yield_now().await,
            Ok(AsyncCall::Sleep(millis)) => Delay::new(Duration::from_millis(millis)).await,
            Err(Error::CoroutineInactive) => break,
            Err(err) => panic!("{}", err),
        }
    }

    Ok(())
}

/// copied from async-std
#[inline]
async fn yield_now() {
    YieldNow(false).await
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    // The futures executor is implemented as a FIFO queue, so all this future
    // does is re-schedule the future back to the end of the queue, giving room
    // for other futures to progress.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
