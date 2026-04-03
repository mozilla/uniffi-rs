{#
 # Builtin cleaner
 #
 # This is used when `disable_java_cleaner` is set in the Config.
 # It doesn't depend on a system `Cleaner` implementation
 # or even the `Cleaner` interface definition, which was added in Java 9.
 #}

interface Cleanable {
    fun clean()
}

class CleanerReference(
    obj: Any,
    queue: java.lang.ref.ReferenceQueue<Any>,
    private val cleanupTask: Runnable,
) : java.lang.ref.PhantomReference<Any>(obj, queue), Cleanable {
    override fun clean() {
        // Remove this from the `liveReferences` set and only run the cleanup task if we actually
        // removed an entry.  This way if multiple threads call `clean()` at the same time, the task
        // only runs once.
        if (CLEANER.liveReferences.remove(this)) {
            this.cleanupTask.run();
        }
    }

}

object CLEANER {
    internal val queue = java.lang.ref.ReferenceQueue<Any>();
    // Number of references currently in the queue.
    //
    // This is used to determine if we should start/stop the thread that removes items from the
    // reference queue.
    internal val cleanerRefCounter = java.util.concurrent.atomic.AtomicInteger(0)

    val liveReferences = java.util.concurrent.ConcurrentHashMap.newKeySet<CleanerReference>()

    fun register(obj: Any, cleanupTask: Runnable): Cleanable {
        val cleanable = CleanerReference(obj, queue, cleanupTask)
        liveReferences.add(cleanable)
        if (cleanerRefCounter.incrementAndGet() == 1) {
            // First reference created, start a new thread.
            startThread()
        }
        return cleanable
    }

    private fun startThread() {
        kotlin.concurrent.thread(isDaemon=true, priority=1) {
            while (true) {
                val reference = queue.remove();
                if (reference is CleanerReference) {
                    reference.clean();
                    if (cleanerRefCounter.decrementAndGet() == 0) {
                        // Last reference cleaned, stop the thread
                        break;
                    }
                }
            }
        }
    }
}
