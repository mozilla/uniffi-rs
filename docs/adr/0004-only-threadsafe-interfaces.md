# Remove support for non-`Send+Sync` interfaces

* Status: proposed
* Deciders: mhammond, rfkelly, travis; proposed: jhugman, dmose, janerik
* Date: 2021-03-31

Technical Story: [Issue 419](https://github.com/mozilla/uniffi-rs/issues/419)

## Context and Problem Statement

[ADR-0003](0003-threadsafe-interfaces.md) introduced support for "thread-safe
interfaces" - possibly leading to the impression that there is such a thing as
non-threadsafe interfaces and confusion about exactly what the attribute means.

However, the entire concept of non-threadsafe interfaces is a mismomer -
everything wrapped by uniffi is thread-safe - the only question is who manages
this thread-safety. Interfaces which are not marked as thread-safe cause uniffi
to wrap the interface in a mutex which is hidden in the generated code and
therefore not obvious to the casual reader.

The [Threadsafe] marker acts as a way for the component author to opt out of
the overhead and blocking behaviour of this mutex, at the cost of opting in to
managing their own locking internally. This ADR proposes that uniffi forces
component authors to explicitly manage that locking in all cases - or to put
this in Rust terms, that all structs supported by uniffi must already be
`Send+Sync`

Note that this ADR will hence-forth use the term `Send+Sync` instead of
"Threadsafe" because it more accurately describes the actual intent and avoids
any misunderstandings that might be caused by using the somewhat broad and
generic "Threadsafe".

## Decision Drivers

* Supporting non-`Send+Sync` structs means uniffi must add hidden locking to make
  them `Send+Sync`. We consider this a "foot-gun" as it may lead to accidentally
  having method calls unexpectedly block for long periods, such as
  [this Fenix bug](https://github.com/mozilla-mobile/fenix/issues/17086)
  (with more details available in [this JIRA ticket](https://jira.mozilla.com/browse/SDK-157).)

* Supporting such structs will hinder uniffi growing in directions that we've
  found are desired in practice, such as allowing structs to use [alternative
  method receivers](https://github.com/mozilla/uniffi-rs/issues/417) or to
  [pass interface references over the FFI](https://github.com/mozilla/uniffi-rs/issues/419).

## Considered Options

* [Option 1] Continue supporting non-`Send+Sync` interfaces while also working
  on the enhancements listed above, but exclude non-`Send+Sync` interfaces from
  such enhancements.

* [Option 2] Immediately deprecate, then remove entirely, support for
  non-`Send+Sync` interfaces.

## Decision Outcome

Chosen option:

* **[Option 2] Immediately deprecate, then remove entirely, support for
  non-`Send+Sync` interfaces.**

This decision was taken because our real world experience tells us that
non-`Send+Sync` interfaces are only useful in toy or example applications (eg,
the nimbus and autofill projects didn't get very far before needing these
capabilities), so the extra ongoing work in supporting these interfaces can not
be justified.

### Positive Consequences

* The locking in all uniffi supported component will more easily
  discoverable - it will be in hand-written rust code and not hidden inside
  generated code. This is a benefit to the developers of the uniffi supported
  component rather than to the consumers of it; while we are considering other
  features to help communicate the lock semantics to such consumers, that is
  beyond the scope of this ADR.

* Opens the door to enhancements that would be impossible for non-`Send+Sync`
  interfaces, and simpler to implement for `Send+Sync` interfaces if support
  for non-`Send+Sync` interfaces did not exist.

* Simpler implementation and documentation.

### Negative Consequences

* All consumers (both inside Mozilla and external) will need to change their
  interfaces to be `Send+Sync`. As an example of what this entails,
  see [this commit](https://github.com/mozilla/uniffi-rs/commit/454dfff6aa560dffad980a9258853108a44d5985)
  which converts the `todolist` example.

* Simple, toy applications may be more difficult to wrap - consumers will not
  be able to defer decisions about `Send+Sync` support and will instead need to
  implement simple locking as demonstrated in [this commit](
  https://github.com/mozilla/uniffi-rs/commit/454dfff6aa560dffad980a9258853108a44d5985).

* Existing applications that are yet to consider how to make their
  implementations `Send+Sync` can not be wrapped until they have.

* The examples which aren't currently marked with the `[Threadsafe]` attribute
  will become more complex as they will all need to implement and explain how
  they achieve being `Send+Sync`.

* The perception that its more difficult to wrap interfaces will lead to less
  adoption of the tool.

## Pros and Cons of the Options

### [Option 1]

* Good, because we don't break anyone.
* Bad, because we believe non-`Send+Sync` interfaces aren't useful in the
  real-world, but we would pay the maintenance cost as though they were.
* Bad, because locking remains hidden and leaves the door open to the same
  gun we have already shot ourselves in the foot with.

### [Option 2]

* Good, because it makes the implementation of desired features easier.
* Good, because it removes a foot-gun and makes locking both explicit and
  visible to the developers of the uniffi-wrapped component.
* Bad, because it breaks existing external consumers - it also breaks a couple
  of internal consumers (for example, [fxa-client](
  https://github.com/mozilla/application-services/blob/f3f0cf6e3386bf3036b074dad3950389cbd05746/components/fxa-client/src/fxa_client.udl#L97)),
  but we believe fixing them is easy and low cost.

## Implications

We know there are external consumers and we know that this will break them.
Therefore, we will commit to the following actions:

* Communicating both this decision and how consumers can work around it as soon
  as possible.

* Noisily deprecate non-`Send+Sync` interfaces so existing consumers are likely
  to see warnings and a link to our documentation when they upgrade.

* Upgrade all internal mozilla consumers as soon as possible so they do not
  issue deprecation warnings. As an example of what this entails,
  see [this PR](https://github.com/mozilla/uniffi-rs/commit/454dfff6aa560dffad980a9258853108a44d5985)
  which converts the `todolist` example to be `Send+Sync`.

* Perform the actual removal as late as possible (ie, until support for non
  `Send+Sync` interfaces actually inhibits our ability to add new features).
  Concretely, the actual removal involves:

  * Making `[Threadsafe]` the default. The attribute will not be immediately
    removed as that would break existing `Send+Sync` components, although we
    will mark it as deprecated and remove it on an aggressive timeline as the
    attribute may be confusing given `Send+Sync` would now be the default.

  * Remove support for generating the `Send+Sync` support in generated rust.
    This will cause rust objects that don't support `Send + Sync` to fail to
    compile.

## Links

* Logical extension of [ADR-0003](0003-threadsafe-interfaces.md)
