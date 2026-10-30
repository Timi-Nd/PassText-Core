"""Exception classes for PassText."""


class PassTextError(Exception):
    """Base class for all PassText errors."""

    def __str__(self) -> str:
        return (
            self.args[0] if self.args and isinstance(self.args[0], str) else self.__class__.__name__
        )

    def __repr__(self) -> str:
        return f"{self.__class__.__name__}({self.__str__()!r})"


class InvalidInputTypeError(PassTextError):
    """Raised when a non-string password value is supplied."""


class InvalidUnicodeError(PassTextError):
    """Raised when the input cannot be represented as valid UTF-8."""


class ProcessingLimitError(PassTextError):
    """Raised when the input exceeds configured processing limits."""


class InvalidConfigurationError(PassTextError):
    """Raised when a configuration value is invalid or inconsistent."""


class ProcessingFailureError(PassTextError):
    """Raised when processing fails unexpectedly despite valid input."""
