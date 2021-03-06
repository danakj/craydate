pub use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, RawWaker, RawWakerVTable, Waker};

/// Tracks a Future whose ownership was given to the executor.
///
/// The Future is boxed in order for the Executor to extend its lifetime.
struct ExecutorOwnedFuture<T>(Pin<Box<dyn Future<Output = T>>>);

impl<T> ExecutorOwnedFuture<T> {
  fn as_mut(&mut self) -> Pin<&mut dyn Future<Output = T>> {
    self.0.as_mut()
  }
}

/// Manager of async tasks. The Executor lives for the life of the program, and is stored as a
/// pointer in Wakers or accessed from within Futures. Because it's accessed through a pointer at
/// arbitrary times, we can not store it as a reference when we would leave the craydate crate. Any
/// waking of a Waker or polling of a Future can leave the crate, so we implement Executor as static
/// functions acting on `*mut Executor` to avoid a `&mut self` reference that would be unsound when
/// the Executor interacts with Wakers or Futures.
#[non_exhaustive]
pub(crate) struct Executor {
  // The main Future is different than other spawned Futures, in that it never completes and thus
  // has an output type of `!`.
  main_future: Option<ExecutorOwnedFuture<!>>,
  // The main function should not be polled until Playdate is ready for arbitrary code to run, which
  // we believe is signalled by the first update_callback(). This tracks that we need to `poll()`
  // the main_future in the next update_callback(). After that it's polled when the given Waker
  // signals.
  first_poll_main: bool,

  // The executor provides async "blocking" tasks, and keeps track of the Wakers that are
  // currently waiting for them.
  //
  // These are waiting for system events.
  pub system_wakers: Vec<Waker>,
}
impl Executor {
  pub fn new() -> Executor {
    Executor {
      main_future: None,
      first_poll_main: false,
      // There will only ever be a single such waker unless we introduce a spawn()
      // or similar function that has a 2nd async function running in tandem with the
      // main function (ie. when it blocks on an async thing).
      system_wakers: Vec::with_capacity(1),
    }
  }

  // Tracks the spawned main Future, but delays polling it until explicitly requested to.
  pub fn set_main_future(exec_ptr: NonNull<Executor>, main: Pin<Box<dyn Future<Output = !>>>) {
    let exec = unsafe { Self::as_mut_ref(exec_ptr) };
    exec.main_future = Some(ExecutorOwnedFuture(main));
    exec.first_poll_main = true;
  }

  pub fn add_waker_for_system_event(exec_ptr: NonNull<Executor>, waker: &Waker) {
    let exec = unsafe { Self::as_mut_ref(exec_ptr) };
    exec.system_wakers.push(waker.clone());
  }

  // A possible future thing:
  // ```
  // fn spawn(_exec_ptr: *mut Executor, _future: Pin<Box<dyn Future<Output = ()>>>) {
  //   Save it in a Vec<ExecutorOwnedFuture> until the next idle time, which is probably the
  //   update_callback(), since when we return up the stack we have to wait for that. We don't
  //   have an idle callback, or timer callback, from Playdate or anything. At that time, poll()
  //   the future, and then just poll() it again when the waker given to the last poll() is woken.
  //   todo!()
  // }
  // ```

  pub fn poll_futures(exec_ptr: NonNull<Executor>) {
    let exec = unsafe { Self::as_mut_ref(exec_ptr) };
    if exec.first_poll_main {
      exec.first_poll_main = false;
      drop(exec);
      let waker = never_return_waker::make_waker(exec_ptr);
      // SAFETY: The Executor reference is dropped before calling poll_main().
      unsafe { Self::poll_main(exec_ptr, waker) }
    }

    // Note: If we had a spawn() function with other Futures given to it, we'd need to poll them
    // here.
  }

  pub fn wake_system_wakers(exec_ptr: NonNull<Executor>) {
    let exec = unsafe { Self::as_mut_ref(exec_ptr) };
    let wakers = core::mem::replace(&mut exec.system_wakers, Vec::with_capacity(1));
    drop(exec);

    for w in wakers {
      // SAFETY: Waking a waker can execute arbitrary code, including going into Executor, so we
      // must not be holding a reference to Executor. Thus we drop() the executor reference above.
      w.wake()
    }
  }

  // SAFETY: The reference must not be alive when leaving the Executor class, including by calling a
  // Waker or a Future. Else it may violate aliasing rules if Exector is re-entered.
  unsafe fn as_mut_ref(exec_ptr: NonNull<Executor>) -> &'static mut Executor {
    &mut *exec_ptr.as_ptr()
  }

  // Polls the main function.
  //
  // SAFETY: The caller must ensure it does not hold a reference to the Executor as this function
  // will create &mut reference to it.
  unsafe fn poll_main(exec_ptr: NonNull<Executor>, waker: Waker) {
    let exec = Self::as_mut_ref(exec_ptr);
    // SAFETY: Get a reference to the main_future which is in the heap, not part of the Executor
    // type directly. Then drop the reference to Executor before calling poll().
    let mut future = core::mem::replace(&mut exec.main_future, None).unwrap();
    drop(exec);

    let _ = future.as_mut().poll(&mut Context::from_waker(&waker));

    // `future` has an output type `!` so poll() definitely returned Poll::Pending. Save the Future
    // to keep running it.
    let exec = Self::as_mut_ref(exec_ptr);
    exec.main_future = Some(future);
  }
}

mod never_return_waker {
  //! Implements a Waker for an ExecutiveOwnedFuture that never returns.
  //!
  //! Since the Future never returns, it never needs to be destroyed. Thus there's no need to
  //! coordinate destruction with the Executor that owns it.
  use super::*;

  #[derive(Clone, Debug)]
  struct WakerData {
    refs: u32,
    exec_ptr: NonNull<Executor>,
  }

  fn clone_fn(data_ptr: *const ()) -> RawWaker {
    unsafe { (*as_data(data_ptr)).refs += 1 };
    RawWaker::new(data_ptr, &VTABLE)
  }
  fn wake_fn(data_ptr: *const ()) {
    // Steal the data_ptr from the Waker being dropped.
    let waker = unsafe { Waker::from_raw(RawWaker::new(data_ptr as *const (), &VTABLE)) };
    // SAFETY: No Executor is held while calling poll_main().
    unsafe { Executor::poll_main((*as_data(data_ptr)).exec_ptr, waker) }

    // Don't change the `data`'s refs or drop it here. This is called when the Waker will not be
    // dropped separately, so the data_ptr won't be freed by the Waker for this function.
  }
  fn wake_by_ref_fn(data_ptr: *const ()) {
    // Clone the Waker and its data.
    let waker = unsafe { Waker::from_raw(clone_fn(data_ptr)) };
    // SAFETY: No Executor is held while calling poll_main().
    unsafe { Executor::poll_main((*as_data(data_ptr)).exec_ptr, waker) }
  }
  fn drop_fn(data_ptr: *const ()) {
    let data = unsafe { &mut *(data_ptr as *mut WakerData) };
    data.refs -= 1;
    if data.refs == 0 {
      unsafe { Box::from_raw(data) };
    }
  }

  fn as_data(data_ptr: *const ()) -> *mut WakerData {
    data_ptr as *mut WakerData
  }

  static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_fn, wake_fn, wake_by_ref_fn, drop_fn);

  pub(crate) fn make_waker(exec_ptr: NonNull<Executor>) -> Waker {
    let data_ptr = Box::into_raw(Box::new(WakerData { refs: 1, exec_ptr }));
    let raw_waker = RawWaker::new(data_ptr as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
  }
}
