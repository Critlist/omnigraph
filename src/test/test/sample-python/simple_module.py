"""Simple Python module for basic parser testing"""

import os
import sys
from pathlib import Path
from typing import Union, Tuple

# Global variables
DEBUG = True
VERSION = "1.0.0"
_internal_cache = {}


def simple_function(name: str) -> str:
    """Simple function with type hints"""
    return f"Hello, {name}!"


def function_with_complexity(data: list, threshold: int = 10) -> dict:
    """Function with control flow complexity"""
    result = {"processed": 0, "errors": 0}
    
    for item in data:
        try:
            if isinstance(item, str):
                if len(item) > threshold:
                    result["processed"] += 1
                else:
                    continue
            elif isinstance(item, (int, float)):
                if item > threshold:
                    result["processed"] += 1
            else:
                result["errors"] += 1
        except Exception:
            result["errors"] += 1
    
    return result


class SimpleClass:
    """Simple class for testing"""
    
    def __init__(self, name: str):
        self.name = name
        self.created_at = "now"
    
    def get_info(self) -> dict:
        """Get class information"""
        return {
            "name": self.name,
            "created_at": self.created_at,
            "type": "SimpleClass"
        }


# Module-level execution
if __name__ == "__main__":
    test_instance = SimpleClass("test")
    print(test_instance.get_info()) 