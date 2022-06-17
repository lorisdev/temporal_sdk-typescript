use neon::prelude::*;
use once_cell::sync::OnceCell;

/// An unhandled error while communicating with the server, considered fatal
pub static TRANSPORT_ERROR: OnceCell<Root<JsFunction>> = OnceCell::new();
/// Thrown after shutdown was requested as a response to a poll function, JS should stop polling
/// once this error is encountered
pub static SHUTDOWN_ERROR: OnceCell<Root<JsFunction>> = OnceCell::new();
/// Something unexpected happened, considered fatal
pub static UNEXPECTED_ERROR: OnceCell<Root<JsFunction>> = OnceCell::new();
/// Used in different parts of the project to signal that something unexpected has happened
pub static ILLEGAL_STATE_ERROR: OnceCell<Root<JsFunction>> = OnceCell::new();

static ALREADY_REGISTERED_ERRORS: OnceCell<bool> = OnceCell::new();

/// This is one of the ways to implement custom errors in neon.
/// Taken from the answer in GitHub issues: https://github.com/neon-bindings/neon/issues/714
pub trait CustomError {
    fn construct<'a, C>(&self, cx: &mut C, args: Vec<Handle<JsValue>>) -> JsResult<'a, JsObject>
    where
        C: Context<'a>;

    fn from_string<'a, C>(&self, cx: &mut C, message: impl Into<String>) -> JsResult<'a, JsObject>
    where
        C: Context<'a>;

    fn from_error<'a, C, E>(&self, cx: &mut C, err: E) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
        E: std::error::Error;
}

// Implement `CustomError` for ALL errors in a `OnceCell`. This only needs to be
// done _once_ even if other errors are added.
impl CustomError for OnceCell<Root<JsFunction>> {
    fn construct<'a, C>(&self, cx: &mut C, args: Vec<Handle<JsValue>>) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        let error = self
            .get()
            .expect("Expected module to be initialized")
            .to_inner(cx);

        // Use `.construct` to call this as a constructor instead of a normal function
        error.construct(cx, args)
    }

    fn from_string<'a, C>(&self, cx: &mut C, message: impl Into<String>) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        let args = vec![cx.string(message.into()).upcast()];
        self.construct(cx, args)
    }

    fn from_error<'a, C, E>(&self, cx: &mut C, err: E) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
        E: std::error::Error,
    {
        self.from_string(cx, format!("{:?}", err))
    }
}

/// This method should be manually called _once_ from JavaScript to initialize the module
/// It expects a single argument, an object with the various Error constructors.
/// This is a very common pattern in Neon modules.
pub fn register_errors(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let res = ALREADY_REGISTERED_ERRORS.set(true);
    if res.is_err() {
        // Don't do anything if errors are already registered
        return Ok(cx.undefined());
    }

    let mapping = cx.argument::<JsObject>(0)?;
    let shutdown_error = mapping
        .get::<JsFunction, _, _>(&mut cx, "ShutdownError")?
        .root(&mut cx);
    let transport_error = mapping
        .get::<JsFunction, _, _>(&mut cx, "TransportError")?
        .root(&mut cx);
    let unexpected_error = mapping
        .get::<JsFunction, _, _>(&mut cx, "UnexpectedError")?
        .root(&mut cx);
    let illegal_state_error = mapping
        .get::<JsFunction, _, _>(&mut cx, "IllegalStateError")?
        .root(&mut cx);

    TRANSPORT_ERROR.get_or_try_init(|| Ok(transport_error))?;
    SHUTDOWN_ERROR.get_or_try_init(|| Ok(shutdown_error))?;
    UNEXPECTED_ERROR.get_or_try_init(|| Ok(unexpected_error))?;
    ILLEGAL_STATE_ERROR.get_or_try_init(|| Ok(illegal_state_error))?;

    Ok(cx.undefined())
}
