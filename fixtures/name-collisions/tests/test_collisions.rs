/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Integration tests for name collision scenarios

use std::sync::Arc;
use uniffi_name_collisions::*;

#[test]
fn test_local_logger() {
    // Test that our local Logger works independently
    let logger = Logger::new();
    logger.log("test message".to_string());
    let messages = logger.get_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0], "test message");
}

#[test]
fn test_local_processor() {
    let processor = LocalProcessor::new("test".to_string());
    let result = processor.process("data".to_string());
    assert_eq!(result, "test: data");
}

#[test]
fn test_processor_trait() {
    // This tests that we can have a trait with the same name as other fixtures
    let processor = LocalProcessor::new("TestProcessor".to_string());
    let result = processor.process("test data".to_string());
    assert_eq!(result, "TestProcessor: test data");
}

#[test]
fn test_namespace_functions() {
    let processed = process_data("input".to_string());
    assert_eq!(processed, "processed: input");
    
    let logged = log_message("log entry".to_string());
    assert_eq!(logged, "collision-fixture: log entry");
}

#[test]
fn test_full_collision_scenario() {
    let result = test_collision_scenario();
    
    // Verify all components are working together
    assert!(result.contains("Local collision test completed"));
    assert!(result.contains("1 messages"));
}

#[test]
fn test_foreign_trait_implementation() {
    // Test implementing the Processor trait from foreign code
    struct ForeignProcessor;
    
    impl Processor for ForeignProcessor {
        fn process(&self, data: String) -> String {
            format!("Foreign implementation: {}", data)
        }
    }
    
    let foreign_proc = Arc::new(ForeignProcessor);
    let result = foreign_proc.process("foreign data".to_string());
    assert!(result.contains("Foreign implementation"));
    assert!(result.contains("foreign data"));
}
