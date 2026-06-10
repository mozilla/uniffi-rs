# frozen_string_literal: true

require 'test/unit'
require 'futures'

class TestFutures < Test::Unit::TestCase
  def test_always_ready
    assert_equal true, Futures.always_ready
  end

  def test_void
    assert_nil Futures.void
  end

  def test_sleep
    start = now()
    Futures.sleep 200

    assert_in_delta 0.2, now - start, 0.1
  end

  def test_sequential_futures
    start = now()
    result_alice = Futures.say_after 100, 'Alice'
    result_bob = Futures.say_after 200, 'Bob'

    assert_operator now - start, :>=, 0.3
    assert_equal result_alice, 'Hello, Alice!'
    assert_equal result_bob, 'Hello, Bob!'
  end

  def test_concurrent_tasks
    start = now()
    alice = Thread.new { Futures.say_after(100, 'Alice') }
    bob = Thread.new { Futures.say_after(200, 'Bob') }
    result_alice = alice.value
    result_bob = bob.value

    delta = now - start

    assert_operator delta, :>=, 0.2
    assert_operator delta, :<, 0.4

    assert_equal result_alice, 'Hello, Alice!'
    assert_equal result_bob, 'Hello, Bob!'
  end

  def test_async_methods
    megaphone = Futures::Megaphone.new
    t0 = now()
    result_alice = megaphone.say_after(200, 'Alice')

    assert_operator now - t0, :>=, 0.2
    assert_equal result_alice, 'HELLO, ALICE!'
  end

  def test_async_constructors
    megaphone = Futures::Megaphone.secondary
    result_alice = megaphone.say_after(0, 'Alice')
    assert_equal result_alice, 'HELLO, ALICE!'

    udl_megaphone = Futures::UdlMegaphone.secondary
    result_udl = udl_megaphone.say_after(0, 'udl')
    assert_equal result_udl, 'HELLO, UDL!'
  end

  def test_async_trait_interface_methods
    traits = Futures.get_say_after_traits
    start = now()
    result1 = traits[0].say_after(100, 'Alice')
    result2 = traits[1].say_after(100, 'Bob')

    assert_operator now - start, :>=, 0.2
    assert_equal result1, 'Hello, Alice!'
    assert_equal result2, 'Hello, Bob!'
  end

  def test_udl_async_trait_interface_methods
    traits = Futures.get_say_after_udl_traits
    t0 = now()
    result1 = traits[0].say_after(100, 'Alice')
    result2 = traits[1].say_after(100, 'Bob')
    t_delta = now - t0

    assert_equal result1, 'Hello, Alice!'
    assert_equal result2, 'Hello, Bob!'

    assert_operator t_delta, :>=, 0.2
  end

  def test_tokio_async_trait_interface_methods
    traits = Futures.get_say_after_tokio_traits

    start = now()

    result1 = traits[0].say_after(100, 'Alice')
    result2 = traits[1].say_after(100, 'Bob')

    assert_operator now - start, :>=, 0.2
    assert_equal result1, 'Hello, Alice (with Tokio)!'
    assert_equal result2, 'Hello, Bob (with Tokio)!'
  end

  def test_foreign_async_trait_interface_methods
    trait_obj = RbAsyncParser.new

    assert_equal Futures.as_string_using_trait(trait_obj, 1, 42), '42'
    assert_equal Futures.try_from_string_using_trait(trait_obj, 1, '42'), 42
    assert_raises(Futures::ParserError::NotAnInt) do
      Futures.try_from_string_using_trait(trait_obj, 1, 'fourty-two')
    end

    assert_raises(Futures::ParserError::UnexpectedError) do
      Futures.try_from_string_using_trait trait_obj, 1, 'force-unexpected-exception'
    end

    Futures.delay_using_trait(trait_obj, 1)
    Futures.try_delay_using_trait(trait_obj, '1')

    assert_raises(Futures::ParserError::NotAnInt) do
      Futures.try_delay_using_trait(trait_obj, 'one')
    end

    completed_delays_before = trait_obj.completed_delays
    Futures.cancel_delay_using_trait trait_obj, 10
    # sleep long enough so that the `delay()` call would finish if it wasn't cancelled.
    sleep 0.1
    # If the task was cancelled, then completed_delays won't have increased
    assert_equal trait_obj.completed_delays, completed_delays_before

    # check that all foreign future handles were released
    assert_equal Futures::UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.size, 0
    assert_equal Futures::UNIFFI_ASYNC_HANDLE_MAP.size, 0
  end

  def test_async_object_param
    megaphone = Futures.new_megaphone()
    t0 = now()
    result_alice = Futures.say_after_with_megaphone(megaphone, 200, 'Alice')

    assert_operator now - t0, :>=, 0.2
    assert_equal result_alice, 'HELLO, ALICE!'
  end

  def test_with_tokio_runtime
    t0 = now()
    result_alice = Futures.say_after_with_tokio(200, 'Alice')

    assert_operator now - t0, :>=, 0.2
    assert_equal result_alice, 'Hello, Alice (with Tokio)!'
  end

  def test_fallible
    assert_equal 42, Futures.fallible_me(false)
    assert_raises(Futures::MyError::Foo) { Futures.fallible_me true }

    megaphone = Futures.new_megaphone
    assert_equal 42, megaphone.fallible_me(false)

    assert_raises(Futures::MyError::Foo) { megaphone.fallible_me(true) }
  end

  def test_fallible_struct
    assert_equal 42, Futures.fallible_struct(false).fallible_me(false)
    assert_raises(Futures::MyError::Foo) { Futures.fallible_struct true }
  end

  def test_record
    result = Futures.new_my_record 'foo', 42

    assert_instance_of Futures::MyRecord, result
    assert_equal result.a, 'foo'
    assert_equal result.b, 42
  end

  def test_greet
    assert_equal 'Hello, World', Futures.greet('World')
  end

  def test_shared_resource_no_cancellation
    Futures.use_shared_resource(
      Futures::SharedResourceOptions.new(release_after_ms: 100, timeout_ms: 1000)
    )
    Futures.use_shared_resource(
      Futures::SharedResourceOptions.new(release_after_ms: 0, timeout_ms: 1000)
    )
  end

  # Test a future that uses a lock and that is cancelled.
  def test_shared_resource_cancellation
    thread = Thread.new do
      Futures.use_shared_resource(
        Futures::SharedResourceOptions.new(release_after_ms: 5000, timeout_ms: 100)
      )
    end

    thread.report_on_exception = false
    sleep 0.05 # let the thread acquire the Tokio mutext
    thread.raise # cancel - drops the Rust future, releasing the mutex guard
    thread.join rescue nil

    # If cancellation freed the future correctly, the mutext is released and this succeeds
    Futures.use_shared_resource(
      Futures::SharedResourceOptions.new(release_after_ms: 0, timeout_ms: 1000)
    )
  end

  def test_deadlock_stress
    50.times do |i|
      threads = 4.times.map do
        t = Thread.new do
          Futures.use_shared_resource(
            Futures::SharedResourceOptions.new(release_after_ms: 5000, timeout_ms: 10)
          )
        end
        t.report_on_exception = false
        t
      end

      sleep 0.01
      threads.each(&:raise) # cancel

      threads.each do |t|
        begin
          t.join(2)
        rescue Exception
          # Expected - thread died from RuntimeError (our raise) or AsyncError::Timeout
          next
        end
        # join returned nil (timeout) - thread is stuck
        if !t.alive?
          next # Thread finished between join timeout and alive? check
        end

        t.kill
        flunk "Deadlock detected in iteration #{i}"
      end
    end
  end

  def test_cancel
    thread = Thread.new { Futures.say_after 200, 'Alice' }
    thread.report_on_exception = false
    sleep 0.1 # let the poll loop start
    thread.raise # interrupt wait_readable -> ensure fires cancel+free
    thread.join rescue nil

    assert !thread.alive?, 'Future was not canceled'

    # No leaked pipe handles
    assert_equal Futures::UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.size, 0
    assert_equal Futures::UNIFFI_ASYNC_HANDLE_MAP.size, 0
  end

  # --- Fiber::Scheduler tests ---
  # These verify that async calls work with Ruby's cooperative fiber concurrency,
  # yielding the fiber on IO wait and resuming when the Rust future completes.
  def test_fiber_concurrent_tasks
    results = []
    t0 = now()

    run_with_scheduler do
      Fiber.schedule { results << Futures.say_after(200, 'Alice') }
      Fiber.schedule { results << Futures.say_after(200, 'Bob') }
    end

    delta = now - t0

    assert_equal 2, results.size
    assert_include results, 'Hello, Alice!'
    assert_include results, 'Hello, Bob!'

    # Both run concurrently on one thread - should complete in ~200ms, not ~400ms
    assert_operator delta, :>=, 0.2
    assert_operator delta, :<, 0.4
  end

  def test_fiber_async_methods
    result = nil

    run_with_scheduler do
      Fiber.schedule { result = Futures.new_megaphone.say_after 100, 'Alice' }
    end

    assert_equal 'HELLO, ALICE!', result
  end

  def test_fiber_async_constructors
    result = nil

    run_with_scheduler do
      Fiber.schedule { result = Futures::Megaphone.secondary.say_after 0, 'Alice' }
    end

    assert_equal 'HELLO, ALICE!', result
  end

  def test_fiber_fallible
    ok_result = nil
    error_class = nil

    run_with_scheduler do
      Fiber.schedule { ok_result = Futures.fallible_me(false) }
      Fiber.schedule do
        Futures.fallible_me true
      rescue StandardError => e
        error_class = e.class
      end
    end

    assert_equal 42, ok_result
    assert_equal Futures::MyError::Foo, error_class
  end

  def now
    Process.clock_gettime Process::CLOCK_MONOTONIC
  end

  # Minimal Fiber::Scheduler for testing (no external gems required).
  # Implements just enough of the scheduler interface to support IO#wait_readable
  # which is used by uniffi_rust_call_async
  class MinimalScheduler
    def initialize
      @readable = {}
      @waiting = []
      @ready = []
    end

    def io_wait(io, events, _timeout = nil)
      @readable[io] = Fiber.current
      Fiber.yield
      events
    end

    def kernel_sleep(duration = nil)
      return unless duration

      @waiting << [Fiber.current, Process.clock_gettime(Process::CLOCK_MONOTONIC) + duration]
      Fiber.yield
    end

    def block(blocker, timeout = nil); end
    def unblock(blocker, fiber); end

    def fiber(&block)
      f = Fiber.new(blocking: false, &block)
      f.resume
      f
    end

    def fiber_interrupt(fiber, exception)
      # 1. Pull the fiber out of any wait state to avoid double-resumption
      @readable.delete_if { |_, f| f == fiber }
      @waiting.delete_if { |f, _| f == fiber }

      # 2. Queue it up to be interrupted in the main loop
      @ready << [fiber, exception]
    end

    def close
      until @readable.empty? && @waiting.empty? && @ready.empty?

        # 1. Process ready/interrupted fibers immediately
        while (pair = @ready.shift)
          f, exception = pair
          # Raise the exception inside the fiber at the point it called Fiber.yield
          f.raise(exception) if f.alive? 
        end

        now = Process.clock_gettime(Process::CLOCK_MONOTONIC)
        @waiting.select { |_, t| t < now }.each do |pair|
          @waiting.delete(pair)
          pair[0].resume
        end

        rds = @readable.keys

        break if rds.empty? && @waiting.empty? && @ready.empty?
        next if rds.empty?

        # If there are no waiting fibers, default to a 0.1s max timeout
        t = @waiting.empty? ? 0.1 : [0.001, @waiting.map { |_, deadline| deadline - now }.min].max
        ready = IO.select(rds, nil, nil, t)

        (ready&.first || []).each do |io|
          f = @readable.delete(io)
          f&.resume
        end
      end
    end
  end

  # Run block with a minimal Fiber::Scheduler, then cleanup.
  def run_with_scheduler
    scheduler = MinimalScheduler.new
    Fiber.set_scheduler scheduler
    yield
    scheduler.close
  ensure
    Fiber.set_scheduler nil
  end

  class RbAsyncParser < Futures::AsyncParser
    attr_accessor :completed_delays

    def initialize
      @completed_delays = 0
    end

    def as_string(delay_ms, value)
      Kernel.sleep(delay_ms / 1000.0)
      value.to_s
    end

    def try_from_string(delay_ms, value)
      Kernel.sleep(delay_ms / 1000.0)

      raise RuntimeError('UnexpectedException') if value == 'force-unexpected-exception'

      begin
        Integer(value)
      rescue ArgumentError
        raise Futures::ParserError::NotAnInt
      end
    end

    def delay(delay_ms)
      Kernel.sleep(delay_ms / 1000.0)
      @completed_delays += 1
    end

    def try_delay(delay_ms)
      begin
        delay_ms = Integer(delay_ms)
      rescue ArgumentError
        raise Futures::ParserError::NotAnInt
      end

      Kernel.sleep(delay_ms / 1000.0)
      @completed_delays += 1
    end
  end
end
