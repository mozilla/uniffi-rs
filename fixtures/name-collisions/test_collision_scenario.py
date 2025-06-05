#!/usr/bin/env python3
"""
Test script to demonstrate UniFFI name collision scenarios.

This script attempts to import and use both the 'name-collisions' and 'callbacks' 
fixtures simultaneously to observe how Python handles the naming conflicts.
"""

import sys
import os

# Add both binding directories to Python path
sys.path.insert(0, '/Users/bgruber/CodingIsFun/uniffi-rs/fixtures/name-collisions/bindings/python')
sys.path.insert(0, '/Users/bgruber/CodingIsFun/uniffi-rs/fixtures/callbacks/bindings/python')

def test_separate_imports():
    """Test importing both modules separately."""
    print("=== Testing Separate Imports ===")
    
    try:
        import name_collisions
        print("✓ Successfully imported name_collisions module")
        print(f"  - Available classes: {[name for name in dir(name_collisions) if not name.startswith('_')]}")
        
        import fixture_callbacks
        print("✓ Successfully imported fixture_callbacks module")
        print(f"  - Available classes: {[name for name in dir(fixture_callbacks) if not name.startswith('_')]}")
        
    except Exception as e:
        print(f"✗ Import failed: {e}")
        return False
    
    return True

def test_foreign_getters_collision():
    """Test using ForeignGetters from both modules."""
    print("\n=== Testing ForeignGetters Collision ===")
    
    try:
        import name_collisions
        import fixture_callbacks
        
        # Both modules have ForeignGetters - let's see what happens
        print("ForeignGetters from name_collisions:", name_collisions.ForeignGetters)
        print("ForeignGetters from fixture_callbacks:", fixture_callbacks.ForeignGetters)
        
        # Check if they're the same object (collision) or different
        if name_collisions.ForeignGetters is fixture_callbacks.ForeignGetters:
            print("⚠️  COLLISION: Both modules reference the same ForeignGetters class!")
        else:
            print("✓ No collision: Each module has its own ForeignGetters class")
            
        # Test method signatures
        print("\nMethod signatures comparison:")
        nc_methods = [method for method in dir(name_collisions.ForeignGetters) if not method.startswith('_')]
        fc_methods = [method for method in dir(fixture_callbacks.ForeignGetters) if not method.startswith('_')]
        
        print(f"name_collisions.ForeignGetters methods: {nc_methods}")
        print(f"fixture_callbacks.ForeignGetters methods: {fc_methods}")
        
        # Check for method name collisions
        common_methods = set(nc_methods) & set(fc_methods)
        if common_methods:
            print(f"⚠️  Method name collisions detected: {common_methods}")
        else:
            print("✓ No method name collisions")
            
    except Exception as e:
        print(f"✗ ForeignGetters test failed: {e}")
        return False
    
    return True

def test_global_function_collision():
    """Test global function collisions."""
    print("\n=== Testing Global Function Collision ===")
    
    try:
        import name_collisions
        import fixture_callbacks
        
        # Both modules might have get_string and get_bool functions
        nc_functions = [name for name in dir(name_collisions) if callable(getattr(name_collisions, name)) and not name.startswith('_')]
        fc_functions = [name for name in dir(fixture_callbacks) if callable(getattr(fixture_callbacks, name)) and not name.startswith('_')]
        
        print(f"name_collisions functions: {nc_functions}")
        print(f"fixture_callbacks functions: {fc_functions}")
        
        # Check for function name collisions
        common_functions = set(nc_functions) & set(fc_functions)
        if common_functions:
            print(f"⚠️  Function name collisions detected: {common_functions}")
            
            # Test specific collision functions if they exist
            if hasattr(name_collisions, 'get_string') and hasattr(fixture_callbacks, 'get_string'):
                print("Testing get_string collision:")
                print(f"  name_collisions.get_string: {name_collisions.get_string}")
                print(f"  fixture_callbacks.get_string: {fixture_callbacks.get_string}")
                
        else:
            print("✓ No function name collisions")
            
    except Exception as e:
        print(f"✗ Global function test failed: {e}")
        return False
    
    return True

def test_star_import_collision():
    """Test what happens with star imports (from module import *)."""
    print("\n=== Testing Star Import Collision ===")
    
    try:
        # This is dangerous but will show us what happens with collisions
        print("Attempting star import from name_collisions...")
        exec("from name_collisions import *")
        
        # Check what's in the global namespace now
        globals_before = set(globals().keys())
        
        print("Attempting star import from fixture_callbacks...")
        exec("from fixture_callbacks import *")
        
        globals_after = set(globals().keys())
        new_globals = globals_after - globals_before
        
        print(f"New globals added: {new_globals}")
        
        if 'ForeignGetters' in globals():
            print(f"ForeignGetters in global namespace: {ForeignGetters}")
            print("⚠️  Star import collision - later import may have overridden earlier one")
        
    except Exception as e:
        print(f"✗ Star import test failed: {e}")
        return False
    
    return True

def main():
    """Run all collision tests."""
    print("UniFFI Name Collision Test Suite")
    print("=" * 50)
    
    success = True
    success &= test_separate_imports()
    success &= test_foreign_getters_collision()
    success &= test_global_function_collision()
    success &= test_star_import_collision()
    
    print("\n" + "=" * 50)
    if success:
        print("✓ All tests completed successfully")
    else:
        print("✗ Some tests failed")
    
    return success

if __name__ == "__main__":
    main()
