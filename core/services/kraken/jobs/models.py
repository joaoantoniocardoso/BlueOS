from enum import Enum
from typing import Any, Optional

from pydantic import BaseModel


class JobMethod(str, Enum):
    GET = "GET"
    HEAD = "HEAD"
    POST = "POST"
    PUT = "PUT"
    PATCH = "PATCH"
    DELETE = "DELETE"


class JobStatus(str, Enum):
    PENDING = "pending"
    FAILED = "failed"


class Job(BaseModel):
    id: str
    route: str
    method: JobMethod
    body: Any
    retries: int = 5
    status: JobStatus = JobStatus.PENDING
    error: Optional[str] = None
