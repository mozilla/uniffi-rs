// Android Cleaner implementation

interface AndroidCleaner {
    fun register(value: Any, cleanUpTask: Runnable): AndroidCleanable

    companion object {
        fun instance(): AndroidCleaner = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            AndroidSystemCleaner()
        } else {
            AndroidJnaCleaner()
        }
    }
}

interface AndroidCleanable {
    fun clean()
}

class AndroidSystemCleanable(
    private val cleanable: Cleaner.Cleanable,
) : AndroidCleanable {
    override fun clean() {
        cleanable.clean()
    }
}

class AndroidJnaCleanable(
    private val cleanable: JnaCleaner.Cleanable,
) : AndroidCleanable {
    override fun clean() {
        cleanable.clean()
    }
}

class AndroidSystemCleaner : AndroidCleaner {
    val cleaner = SystemCleaner.cleaner()

    override fun register(value: Any, cleanUpTask: Runnable): AndroidCleanable {
        return AndroidSystemCleanable(cleaner.register(value, cleanUpTask))
    }
}

class AndroidJnaCleaner : AndroidCleaner {
    val cleaner = JnaCleaner.getCleaner()

    override fun register(value: Any, cleanUpTask: Runnable): AndroidCleanable {
        return AndroidJnaCleanable(cleaner.register(value, cleanUpTask))
    }
}
