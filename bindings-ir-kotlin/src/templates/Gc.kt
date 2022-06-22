// It's not possible to force Kotlin to GC, but hopefully this should work in well enough to make our test suites pass
System.gc()
System.runFinalization()
Thread.sleep(1000)
System.gc()
System.runFinalization()
Thread.sleep(1000)
System.gc()
System.runFinalization()
Thread.sleep(1000)

