use crate::callbacks::Callbacks;

pub enum NoNull {}
pub enum AllowNull {}
pub enum Unconstructed {}
pub enum WithCallacks {}
pub enum Constructed {}

/// A builder pattern to construct a callback that will later be called when `SystemEvent::Callback`
/// fires. Connects a closure to a `Callbacks` object which can later run the closure.
pub struct CallbackBuilder<
  'a,
  T = (),
  F: Fn(T) + 'static = fn(T),
  Rule = AllowNull,
  State = Unconstructed,
> {
  callbacks: Option<&'a mut Callbacks<T>>,
  cb: Option<F>,
  _marker: core::marker::PhantomData<(&'a u8, T, F, Rule, State)>,
}
impl<'a> CallbackBuilder<'a, (), fn(()), AllowNull, Unconstructed> {
  /// A null callback, which is used to specify a callback should not be set, or should be removed.
  pub fn none() -> CallbackBuilder<'a, (), fn(()), AllowNull, Constructed> {
    CallbackBuilder {
      callbacks: None,
      cb: None,
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, T, F: Fn(T) + 'static, Rule> CallbackBuilder<'a, T, F, Rule, Unconstructed> {
  /// Attach a `Callbacks` object to this builder, that will hold the closure.
  pub fn with(callbacks: &'a mut Callbacks<T>) -> CallbackBuilder<'a, T, F, Rule, WithCallacks> {
    CallbackBuilder {
      callbacks: Some(callbacks),
      cb: None,
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, T, F: Fn(T) + 'static, Rule> CallbackBuilder<'a, T, F, Rule, WithCallacks> {
  /// Attach a closure to this builder, which will be held in the `Callbacks` object and called via
  /// that same `Callbacks` object.
  pub fn call(self, cb: F) -> CallbackBuilder<'a, T, F, Rule, Constructed> {
    CallbackBuilder {
      callbacks: self.callbacks,
      cb: Some(cb),
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, T, F: Fn(T) + 'static, Rule> CallbackBuilder<'a, T, F, Rule, Constructed> {
  pub(crate) fn into_inner(self) -> Option<(&'a mut Callbacks<T>, F)> {
    self.callbacks.zip(self.cb)
  }
}

/// A `CallbackBuilder` which includes an argument passed from the system to the callback.
pub struct CallbackBuilderWithArg<
  'a,
  Arg = (),
  T = (),
  F: Fn(Arg, T) + 'static = fn(Arg, T),
  Rule = AllowNull,
  State = Unconstructed,
> {
  callbacks: Option<&'a mut Callbacks<T>>,
  cb: Option<F>,
  _marker: core::marker::PhantomData<(&'a u8, Arg, T, F, Rule, State)>,
}
impl<'a> CallbackBuilderWithArg<'a, (), (), fn((), ()), AllowNull, Unconstructed> {
  /// A null callback, which is used to specify a callback should not be set, or should be removed.
  pub fn none() -> CallbackBuilderWithArg<'a, (), (), fn((), ()), AllowNull, Constructed> {
    CallbackBuilderWithArg {
      callbacks: None,
      cb: None,
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, Arg, T, F: Fn(Arg, T) + 'static, Rule> CallbackBuilderWithArg<'a, Arg, T, F, Rule, Unconstructed> {
  /// Attach a `Callbacks` object to this builder, that will hold the closure.
  pub fn with(callbacks: &'a mut Callbacks<T>) -> CallbackBuilderWithArg<'a, Arg, T, F, Rule, WithCallacks> {
    CallbackBuilderWithArg {
      callbacks: Some(callbacks),
      cb: None,
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, Arg, T, F: Fn(Arg, T) + 'static, Rule> CallbackBuilderWithArg<'a, Arg, T, F, Rule, WithCallacks> {
  /// Attach a closure to this builder, which will be held in the `Callbacks` object and called via
  /// that same `Callbacks` object.
  pub fn call(self, cb: F) -> CallbackBuilderWithArg<'a, Arg, T, F, Rule, Constructed> {
    CallbackBuilderWithArg {
      callbacks: self.callbacks,
      cb: Some(cb),
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, Arg, T, F: Fn(Arg, T) + 'static, Rule> CallbackBuilderWithArg<'a, Arg, T, F, Rule, Constructed> {
  pub(crate) fn into_inner(self) -> Option<(&'a mut Callbacks<T>, F)> {
    self.callbacks.zip(self.cb)
  }
}
