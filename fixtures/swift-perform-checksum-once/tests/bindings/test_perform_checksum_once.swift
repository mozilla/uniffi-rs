import CoreFoundation
import swift_perform_checksum_once

let entity = newStruct()
// The checksum is performed the first time a Rust method is called
_ = fn0(arg: entity)

// So we measure time between second and 10,000th calls
_ = fn1(arg: entity)
let startTime = CFAbsoluteTimeGetCurrent()
for _ in 0..<10000 {
  _ = fn2(arg: entity)
}

let elapsedTime = CFAbsoluteTimeGetCurrent() - startTime
assert(elapsedTime < 1, "Elapsed time should be less than 1 second, but is: \(elapsedTime)")
