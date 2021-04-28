import Foundation
import chronological

// Test passing timestamp and duration while returning timestamp
assert(try! add(a: Date.init(timeIntervalSince1970: 100.01), b: 1.01) == Date.init(timeIntervalSince1970: 101.02), "add duration")

// Test passing timestamp while returning duration (note precision error)
assert(try! diff(a: Date.init(timeIntervalSince1970: 101.03), b: Date.init(timeIntervalSince1970: 100.01)) == 1.019999981, "diff dates")

// Test exceptions are propagated
do {
    let _ = try diff(a: Date.init(timeIntervalSince1970: 100), b: Date.init(timeIntervalSince1970: 101))
    fatalError("Should have thrown a TimeDiffError exception!")
} catch ChronologicalError.TimeDiffError {
    // It's okay!
}

// Test that rust timestamps behave like swift timestamps
let swiftBefore = Date.init()
let rustNow = now()
let swiftAfter = Date.init()

assert(swiftBefore <= rustNow)
assert(swiftAfter >= rustNow)