import json
import re
from typing import Any, Optional

from pydantic import BaseModel, validator
from pydantic.fields import ModelField

from manifest.models import ExtensionVersion, RepositoryEntry
from settings import ExtensionSettings

# Matches the reference grammar: https://github.com/distribution/reference/blob/main/reference.go
IDENTIFIER_REGEX = re.compile(r"[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+")
DOCKER_DOMAIN_REGEX = re.compile(r"[A-Za-z0-9]([A-Za-z0-9.-]*[A-Za-z0-9])?(:[0-9]+)?")
_DOCKER_PATH_PART = r"[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*"
DOCKER_PATH_REGEX = re.compile(rf"{_DOCKER_PATH_PART}(/{_DOCKER_PATH_PART})*")


class ExtensionSourceAuth(BaseModel):
    username: str
    password: str


class ExtensionSource(BaseModel):
    identifier: str
    tag: str
    name: str
    docker: str
    enabled: bool
    permissions: str
    user_permissions: str = ""
    auth: Optional[ExtensionSourceAuth] = None

    @staticmethod
    def from_settings(settings: ExtensionSettings) -> "ExtensionSource":
        return ExtensionSource(
            identifier=settings.identifier,
            tag=settings.tag,
            name=settings.name,
            docker=settings.docker,
            enabled=settings.enabled,
            permissions=settings.permissions,
            user_permissions=settings.user_permissions,
        )

    @staticmethod
    def from_repository_version(entry: RepositoryEntry, version: ExtensionVersion) -> "ExtensionSource":
        return ExtensionSource(
            identifier=entry.identifier,
            tag=version.tag,
            name=entry.name,
            docker=entry.docker,
            enabled=False,
            permissions=json.dumps(version.permissions),
            user_permissions="",
        )


class ExtensionInstallSource(ExtensionSource):
    """Manual-install body. Separate from ExtensionSource so already-installed values still list."""

    @validator("identifier", "docker", "tag")
    @classmethod
    def reject_whitespace(cls, value: str) -> str:
        if not value:
            raise ValueError("must not be empty")
        if re.search(r"\s", value):
            raise ValueError("must not contain whitespace")
        return value

    @validator("identifier")
    @classmethod
    def reject_unaddressable_identifier(cls, value: str) -> str:
        if not IDENTIFIER_REGEX.fullmatch(value):
            raise ValueError("must be two words separated by a period, using only letters, digits, '_' and '-'")
        return value

    @validator("docker")
    @classmethod
    def reject_invalid_docker(cls, value: str) -> str:
        domain, separator, path = value.partition("/")
        if not separator:
            domain, path = "", value
        if domain and not DOCKER_DOMAIN_REGEX.fullmatch(domain):
            raise ValueError("has an invalid registry domain")
        if not DOCKER_PATH_REGEX.fullmatch(path):
            raise ValueError("must be a lowercase image name without a tag")
        return value

    @validator("permissions", pre=True)
    @classmethod
    def null_permissions_mean_empty(cls, value: Any) -> Any:
        # Some catalog versions declare "permissions": null, which would crash Extension.start.
        return "{}" if value == "null" else value

    @validator("permissions", "user_permissions")
    @classmethod
    def reject_invalid_permissions(cls, value: str, field: ModelField) -> str:
        if field.name == "user_permissions" and not value:
            return value
        try:
            parsed = json.loads(value)
        except ValueError as error:
            raise ValueError("must be a JSON object") from error
        if not isinstance(parsed, dict):
            raise ValueError("must be a JSON object")
        return value

    @validator("name")
    @classmethod
    def strip_name(cls, value: str) -> str:
        value = value.strip()
        if not value:
            raise ValueError("must not be empty")
        if len(value) > 128:
            raise ValueError("must fit within 128 characters")
        return value
