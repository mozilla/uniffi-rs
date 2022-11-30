import uniffi_futures

// private let CALL_SUCCESS: Int8 = 0
// private let CALL_ERROR: Int8 = 1
// private let CALL_PANIC: Int8 = 2

// private extension RustCallStatus {
//     init() {
//         self.init(
//             code: CALL_SUCCESS,
//             errorBuf: RustBuffer(
//                 capacity: 0,
//                 len: 0,
//                 data: nil
//             )
//         )
//     }
// }

// private func rustCall<T>(_ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T {
//     try makeRustCall(callback, errorHandler: {
//         $0.deallocate()
//         return UniffiInternalError.unexpectedRustCallError
//     })
// }

// private func rustCallWithError<T, F: FfiConverter>
// (_ errorFfiConverter: F.Type, _ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T
//     where F.SwiftType: Error, F.FfiType == RustBuffer
// {
//     try makeRustCall(callback, errorHandler: { try errorFfiConverter.lift($0) })
// }

// private func makeRustCall<T>(_ callback: (UnsafeMutablePointer<RustCallStatus>) -> T, errorHandler: (RustBuffer) throws -> Error) throws -> T {
//     var callStatus = RustCallStatus()
//     let returnedVal = callback(&callStatus)
//     switch callStatus.code {
//     case CALL_SUCCESS:
//         return returnedVal

//     case CALL_ERROR:
//         throw try errorHandler(callStatus.errorBuf)

//     case CALL_PANIC:
//         // When the rust code sees a panic, it tries to construct a RustBuffer
//         // with the message.  But if that code panics, then it just sends back
//         // an empty buffer.
//         if callStatus.errorBuf.len > 0 {
//             throw UniffiInternalError.rustPanic(try FfiConverterString.lift(callStatus.errorBuf))
//         } else {
//             callStatus.errorBuf.deallocate()
//             throw UniffiInternalError.rustPanic("Rust panic")
//         }

//     default:
//         throw UniffiInternalError.unexpectedRustCallStatusCode
//     }
// }

func foo() async -> Bool {
 	let rustFuture = _uniffi_uniffi_futures_always_ready_910a()
	
 	return true

// 	// return await withCheckedContinuation { continuation in
// 	// 	func waker(env: *void) {
// 	// 		env = env as &Env
// 	// 		rustFuture = env.rustFuture
// 	// 		continuation = env.continuation

// 	// 		polledResult = NULL
// 	// 		isReady = ffi_rust_future__poll(rustFuture, waker, env as *void, &mut pollResult)

// 	// 		if isReady {
// 	// 			ffi_rust_future__drop(rustFuture)
// 	// 			continuation.resume(with: lift(polledResult))
// 	// 		}
// 	// 	}
		
// 	// 	struct Env {
// 	// 		rustFuture: rustFuture,
// 	// 		continuation: continuation,
// 	// 	}
		
// 	// 	waker(&env as *void)
// 	// }
}

print("Hello, World!")
print(greet(who: "Gordon"))
