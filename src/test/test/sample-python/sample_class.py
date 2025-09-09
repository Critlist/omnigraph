"""
Sample Python module for testing the parser
Contains various Python constructs to verify AST extraction
"""

from typing import List, Optional, Dict, Iterator
from dataclasses import dataclass, field
import asyncio
from abc import ABC, abstractmethod


class BaseClass(ABC):
    """Base class with abstract methods"""
    
    def __init__(self, name: str):
        self.name = name
        self._private_var = "private"
        self.__very_private = "very private"
    
    @abstractmethod
    def abstract_method(self) -> str:
        """Abstract method to be implemented by subclasses"""
        pass
    
    @property
    def display_name(self) -> str:
        """Property getter for display name"""
        return self.name.upper()


@dataclass
class ConcreteClass(BaseClass):
    """Concrete implementation with decorators and type hints"""
    
    value: int = 0
    items: List[str] = field(default_factory=list)
    
    def __init__(self, name: str, value: int = 42):
        super().__init__(name)
        self.value = value
    
    def abstract_method(self) -> str:
        """Implementation of abstract method"""
        return f"Concrete implementation for {self.name}"
    
    @staticmethod
    def static_utility(data: Dict[str, int]) -> int:
        """Static method for utility calculations"""
        return sum(data.values())
    
    @classmethod
    def from_dict(cls, data: dict) -> 'ConcreteClass':
        """Class method factory"""
        return cls(data.get('name', 'default'), data.get('value', 0))
    
    async def async_operation(self, delay: float = 1.0) -> Optional[str]:
        """Async method demonstrating async/await patterns"""
        await asyncio.sleep(delay)
        return f"Async result for {self.name}"
    
    def generator_method(self) -> Iterator[int]:
        """Generator method using yield"""
        for i in range(self.value):
            yield i * 2


# Module-level functions
def process_data(items: List[str], filter_func=None) -> List[str]:
    """Process list of items with optional filter"""
    if filter_func is None:
        return items
    return [item for item in items if filter_func(item)]


async def async_processor(data: Dict[str, object]) -> bool:
    """Async function at module level"""
    try:
        # Complex logic with nested control structures
        if not data:
            return False
        
        for key, value in data.items():
            if isinstance(value, str):
                continue
            elif isinstance(value, (int, float)):
                if value < 0:
                    raise ValueError(f"Negative value for {key}")
            else:
                print(f"Unknown type for {key}: {type(value)}")
        
        return True
    except Exception as e:
        print(f"Error processing data: {e}")
        return False


# Module-level variables
CONSTANT_VALUE = 42
_private_module_var = "module private"
global_config = {
    "debug": True,
    "max_retries": 3,
    "timeout": 30.0
}


# Decorator definitions
def timing_decorator(func):
    """Decorator for timing function execution"""
    def wrapper(*args, **kwargs):
        import time
        start = time.time()
        result = func(*args, **kwargs)
        end = time.time()
        print(f"{func.__name__} took {end - start:.2f} seconds")
        return result
    return wrapper


@timing_decorator
def decorated_function(x: int, y: int = 10) -> int:
    """Function with decorator and default parameters"""
    return x * y + (x ** 2)


# Complex class with multiple inheritance and mixins
class MixinClass:
    """Mixin providing additional functionality"""
    
    def mixin_method(self) -> str:
        return "From mixin"


class ComplexClass(BaseClass, MixinClass):
    """Class demonstrating multiple inheritance"""
    
    def __init__(self, name: str, complexity: int = 1):
        super().__init__(name)
        self.complexity = complexity
    
    def abstract_method(self) -> str:
        """Implementation with complex logic"""
        if self.complexity > 10:
            return "High complexity"
        elif self.complexity > 5:
            return "Medium complexity"
        else:
            return "Low complexity"
    
    def method_with_nested_functions(self, data: List[int]) -> Dict[str, int]:
        """Method with nested function definitions"""
        
        def inner_processor(item: int) -> int:
            return item * 2 if item > 0 else 0
        
        def inner_filter(item: int) -> bool:
            return item % 2 == 0
        
        processed = [inner_processor(x) for x in data if inner_filter(x)]
        return {
            "count": len(processed),
            "sum": sum(processed),
            "max": max(processed) if processed else 0
        } 