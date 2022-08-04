We've been talking for awhile about "Async UniFFI", but it's never been clear exactly what that means.  A lot of our async use cases cover very different situations, for example:

  - Creating a thread-pool on the foreign language side that we make blocking Rust calls from.  This is the main async use case in practice today, and there has been discussion about supporting it better by auto-generating decorator classes.
  - Making a call into Rust that starts a tokio-based network request and getting a Promise/Future back on the foreign side.
  - Making an async call from Rust into the foreign side, suspending the Rust function until we have a result (JEXL evaluation)
  - Making a blocking call from a Rust-controlled thread into the foreign side (e.g. a Rust thread calling into viaduct).

There's a lot of possibilities, but maybe we can group all uses cases using 4 dimensions:

  - Normal call vs CallbackInterace call.  Are we calling from the foreign side into Rust or from Rust into the foreign side?
  - Rust thread vs Foreign Thread.  Who created/controls the thread the call is coming from?
  - Same thread vs Thread switch.  Do we want the work to happen on a different thread?
  - Blocking vs asynchronous.  Does source thread block while the work is happening?  For our purposes, we can consider a function that returns a Future/Promise as async.

Using those dimensions, we can map out the use cases:


## Normal call / Foreign thread / Same thread / Blocking

Normal use, already fairly well supported, but we've discussed adding decorators as a way to reduce
the boilerplate needed.

## Normal call / Foreign thread / Same thread / Async

This only makes sense is if the Rust code then awaits an async call back into the foreign code using
a `CallbackInterface`.  But I think there are some valid use cases here:
  - Initialization.  For example, if we wanted to do something in desktop after Nimbus was fully
    initialized, then could make an async call to `Nimbus.init()` which then makes async calls to
    `Jexl.eval()`.
  - Network requests where the HTTP request happens on the foreign side, then gets parsed on the
    Rust side, then the parsed result is returned to the foreign side.

## Normal call / Foreign thread / Thread switch / Blocking

This means the foreign code wants to execute something on a Rust thread.  This one seems unlikely to
me, but here's one possible use case: Rust starts up a tokio event loop, then
the foreign code blocks on an HTTP request that runs in that event loop.

## Normal call / Foreign thread / Thread switch / Async

This is basically the same as the previous section, except the foreign side executes the call async
rather than blocking on it which seems more likely in the real world.

## Normal call / Rust Thread / * / *

In general, it's not safe for the foreign code to run on a Rust-controlled thread.  Some languages
may support it, but it's definitely not going to work on Desktop JS.  I think we should consider
this invalid.

## CallbackInterface / Foreign thread / Same thread / Blocking

This is a normal CallbackInterace call and is already supported.

## CallbackInterface / Foreign thread / Same thread / Async

Async call from Rust to the foreign language.  Some use cases:
   - Nimbus JEXL evaluation
   - Async viaduct call

Since this is happening on a foreign thead, it means there's a foreign caller that's waiting for a
return.  There's a couple ways to handle that:

  - Rust returns void, which is the current plan for Nimbus.  This doesn't mean it's a no-op, since
    when the async calls resolve the Rust code will continue to do work.
  - The Rust call is async, so it returns a Future/Promise (this is the other side of the
    `async Nimbus.init()` use case described above).

## CallbackInterface / Foreign thread / Thread switch / * and CallbackInterface / Rust Thread / Same thread / *

This is invalid because it's not safe for foreign code to run on a Rust thread (see above).

## CallbackInterface / Rust Thread / Thread switch / Blocking

A Rust thread wants to make a blocking call into the foreign code, which requires a thread switch.
For example, Rust creates a thread pool of workers.  Sometimes those workers want to make network
calls, so they make a viaduct call.  Since this is happening in a thread pool, it's fine for the
thread to block on the result.

## CallbackInterface / Rust Thread / Thread switch / Async

This is the same as the last one, except the call is async.  The main use case is probably the same
as the previous section, except when Rust is using an event loop rather than a thread pool.

# New features

Based on the above analysis, I believe there are 2 features that we should consider adding:

  - Rust code making an async call into a CallbackInterface / Foreign code
    making an async into Rust. I think this could be implemented by having
    UniFFI generate something like the hand-written demo from #1252.
  - Handling CallbackInterface calls from a Rust-based thread. I'm not exactly sure how this would work, there's at least 2 options here:
    - Handle everything on the foreign side. In the generate code we currently register a callback to invoke a CallbackInterface call. We might be able to update that code so that it schedules the call to run on the correct thread. But this assumes that it's safe to call that callback on the Rust thread. Is that true for all of our current languages? Are we okay with adding this requirement for future languages?
    - Use a queue plus a waker. Push the CallbackInterface call to a queue, signal the foreign side (maybe writing a byte to a pipe or socket), then the foreign side would wake up and try to read from the queue. This seems more complicated than the first system, but might let us support more foreign languages.


