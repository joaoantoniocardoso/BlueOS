from typing import Any, Callable

from fastapi import HTTPException, status
from loguru import logger


def internal_server_error(endpoint: Callable[..., Any], error: Exception) -> HTTPException:
    logger.opt(exception=error).error(f"Unexpected error in route {endpoint.__module__}.{endpoint.__qualname__}")
    return HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail=str(error))
