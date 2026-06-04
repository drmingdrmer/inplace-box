//! Verify that an `InplaceBox<dyn Future>` whose inner future holds a `&mut`
//! reference across an await point does not trigger Miri's Stacked/Tree Borrows
//! UB.
//!
//! Note: Previously, the function-entry retag of `&mut InplaceBox` covered the
//! storage buffer and invalidated the saved reference.

use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use inplace_box::InplaceBox;

/// Minimal single-threaded executor: polls `fut` to completion.
///
/// This is to prevent including heavy dependencies like `tokio` just for
/// testing the Miri UB scenario.
//
// `Waker::noop()` (stable since 1.85) exceeds the crate's declared MSRV, but
// test code is not part of the consumer-facing MSRV surface.
#[allow(clippy::incompatible_msrv)]
fn block_on<F: Future>(mut fut: F) -> F::Output {
    let mut cx = Context::from_waker(Waker::noop());
    // SAFETY: `fut` is never moved after this point.
    let mut pinned = unsafe { Pin::new_unchecked(&mut fut) };
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {}
        }
    }
}

/// A future that yields once (returns `Pending` on first poll, then `Ready`).
struct YieldOnce(bool);

impl Future for YieldOnce {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Simple test of the executor w/o `InplaceBox`.
#[test]
fn future_mut_ref_across_await() {
    let result = block_on(async {
        let mut value = 1_u32;
        let r: &mut u32 = &mut value;

        // Yield so the future is suspended with `r` live in its state.
        YieldOnce(false).await;

        // On resumption the saved `r` must still be valid.
        *r += 1;
        *r
    });

    assert_eq!(result, 2);
}

/// Same scenario but wrapped in `InplaceBox<dyn Future>`, which is the case
/// that previously triggered the Miri violation via the wide `&mut InplaceBox`
/// retag invalidating the inner future's saved `&mut`.
#[test]
fn inplace_box_future_mut_ref_across_await() {
    let fut = InplaceBox::<dyn Future<Output = u32>, 128>::new(async {
        let mut value = 1_u32;
        let r: &mut u32 = &mut value;

        YieldOnce(false).await;

        *r += 1;
        *r
    });

    let result = block_on(fut);
    assert_eq!(result, 2);
}
