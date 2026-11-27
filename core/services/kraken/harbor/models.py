from datetime import datetime

from pydantic import BaseModel


class ContainerModel(BaseModel):
    name: str
    image: str
    image_id: str
    status: str
    started_at: datetime
    uptime_seconds: float


class ContainerUsageModel(BaseModel):
    cpu: float
    memory: float | str
    disk: int | str
