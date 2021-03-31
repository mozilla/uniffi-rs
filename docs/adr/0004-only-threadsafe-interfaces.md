# Remove support for non-threadsafe interfaces

* Status: proposed
* Deciders: mhammond, proposed: rfkelly, jhugman, dmose, travis, janerik
* Date: 2021-03-31

Technical Story: [Issue 419](https://github.com/mozilla/uniffi-rs/issues/419)

## Context and Problem Statement

[ADR-0003](0003-threadsafe-interfaces.md) introduced support for "thread-safe
interfaces" - possibly leading to the impression that there is such a thing as
non-threadsafe interfaces.

However, the entire concept of non-threadsafe interfaces is a mismomer -
everything wrapped by uniffi is thread-safe - the only question is who manages
this thread-safety. Interfaces which are not marked as thread-safe cause uniffi
to wrap the interface in a mutex which is hidden in the generated code and
therefore not obvious to the casual reader.

## Decision Drivers

* Non-threadsafe structs are considered a "foot-gun", leading to accidentally
  introduce issues like [this Fenix bug](https://github.com/mozilla-mobile/fenix/issues/17086)
  (with more details available in [this JIRA ticket](https://jira.mozilla.com/browse/SDK-157).)

* Supporting such structs will hinder uniffi growing in directions that we've
  found are desired in practice, such as allowing structs to use [alternative
  method receivers](https://github.com/mozilla/uniffi-rs/issues/417) or to
  [pass interface references over the FFI](https://github.com/mozilla/uniffi-rs/issues/419).

## Considered Options

* [Option 1] Continue supporting non-threadsafe interfaces while also working
  on the enhancements listed above, but exclude non-threadsafe interfaces from
  such enhancements.

* [Option 2] Immediately deprecate, then remove entirely, support for
  non-threadsafe interfaces.

## Decision Outcome

Chosen option:

* **[Option 2] Immediately deprecate, then remove entirely, support for
  non-threadsafe interfaces.**

This decision was taken because our real world experience tells us that
non-threadsafe interfaces are only useful in toy or example applications (eg,
the nimbus and autofill projects didn't get very far before needing these
capabilities), so the extra ongoing work in supporting these interfaces can not
be justified.

### Positive Consequences

* The locking in all uniffi supported applications will be clear.

* Opens the door to enhancements that would be impossible for non-threadsafe
  interfaces, and simpler to implement for threadsafe interfaces if
  non-threadsafe interfaces did not exist.

* Simpler implementation and documentation.

### Negative Consequences

* All consumers (both inside Mozilla and external) will need to change their
  interfaces to support thread-safety.

* Simple, toy applications will be more difficult to wrap - consumers will not
  be able to defer decisions about thread-safety.

* Existing applications that are yet to consider thread-safety can not be
  wrapped until they have.

* The examples will become more complex as they will all need to implement and
  explain how they achieve thread-safety.

* The perception that its more difficult to wrap interfaces will lead to less
  adoption of the tool.

## Pros and Cons of the Options

### [Option 1]

* Good, because we don't break anyone.
* Bad, because we believe non-threadsafe interfaces aren't useful in the
  real-world, but we would pay the maintenance cost as though they were.
* Bad, because locking remains hidden and leaves the door open to the same
  gun we have already shot ourselves in the foot with.

### [Option 2]

* Good, because it makes the implementation of desired features easier.
* Good, because it removes a foot-gun and makes locking both explicit and
  visible.
* Bad, because it breaks existing external consumers - it also breaks a couple
  of internal consumers, but we believe fixing them is easy and low cost.

## Implications

We know there are external consumers and we know that this will break them.
Therefore, we will commit to the following actions:

* Communicating both this decision and how consumers can work around it as soon
  as possible.

* Noisily deprecate non-threadsafe interfaces so existing consumers are likely
  to see warnings and a link to our documentation when they upgrade.

* Upgrade all internal mozilla consumers as soon as possible so they do not
  issue deprecation warnings.

* Perform the actual removal as late as possible (ie, until support for non
  threadsafe interfaces actually inhibits our ability to add new features)

## Links

* Logical extension of [ADR-0003](0003-threadsafe-interfaces.md)
