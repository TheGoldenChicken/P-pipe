from enum import Enum

# STATUS = Exit Code
class RequestStatus(Enum):
    CORRECT = 0
    # 1 is reserved for Python general errors, errors with execution
    # 2 is typically reserved for syntax erorrs, not related to specific input...
    SYNTAX_ERROR = 3 
    PARTIAL_CORRECT = 4
    INCORRECT = 5
    PENDING = 6
    DEADLINE_EXCEEDED = 7
