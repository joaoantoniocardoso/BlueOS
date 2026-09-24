import asyncio
import json
import os
import re
from concurrent.futures import ThreadPoolExecutor
from typing import Any, Callable

import fastapi
import zenoh
from commonwealth.utils import blueos_idl
from fastapi.routing import APIRoute
from loguru import logger

from .Singleton import Singleton

PARAM_REGEX = r"{[a-zA-Z0-9_]+}"

HTTP_GATEWAY_JSON_ENCODING = zenoh.Encoding.APPLICATION_JSON


class ZenohSession(metaclass=Singleton):
    session: zenoh.Session | None = None
    config: zenoh.Config
    _executor: ThreadPoolExecutor | None = None
    _liveliness_token: Any | None = None
    _service_name: str | None = None

    def __init__(self, service_name: str) -> None:
        if self.session is not None:
            return

        self._service_name = service_name
        self.zenoh_config(service_name)
        self.session = zenoh.open(self.config)
        self._register_standard_service_keys(service_name)

        self._executor = ThreadPoolExecutor(
            max_workers=4,
            thread_name_prefix="zenoh-",
        )

    def _register_standard_service_keys(self, service_name: str) -> None:
        if self.session is None:
            return
        try:
            blueos_idl.ensure_idl_loaded()
        except FileNotFoundError as error:
            logger.warning(
                "IDL interfaces missing; skipping Zenoh liveliness and info registration: {}",
                error,
            )
            return
        liveliness_key = blueos_idl.service_liveliness_key(service_name)
        self._liveliness_token = self.session.liveliness().declare_token(liveliness_key)

        build = os.environ.get("GIT_DESCRIBE_TAGS", "")
        version = build.split("-", maxsplit=1)[0] if build else "0.0.0"
        service_info = {
            "name": service_name,
            "version": version,
            "build": build,
            "capabilities": [],
        }
        info_key = blueos_idl.info_query_key(service_name)
        info_payload = blueos_idl.encode("blueos_msgs/msg/ServiceInfo", service_info)
        info_encoding = blueos_idl.cdr_encoding("blueos_msgs/msg/ServiceInfo")

        def info_handler(query: zenoh.Query) -> None:
            query.reply(info_key, info_payload, encoding=info_encoding)

        self.session.declare_queryable(info_key, info_handler)

    def submit_to_executor(self, func: Callable[..., Any]) -> None:
        if self._executor is None:
            logger.warning("Zenoh session executor is not available, task will not be initialized.")
            return
        try:
            self._executor.submit(func)
        except Exception as error:
            logger.error(f"Error submitting task to zenoh session executor: {error}")

    def close(self) -> None:
        if self._liveliness_token is not None:
            try:
                self._liveliness_token.undeclare()  # type: ignore[no-untyped-call]
            except Exception:
                pass
            self._liveliness_token = None
        if self.session:
            self.session.close()  # type: ignore[no-untyped-call]
            self.session = None
        if self._executor:
            self._executor.shutdown(wait=False, cancel_futures=True)
            self._executor = None

    def zenoh_config(self, service_name: str) -> None:
        configuration = {
            "mode": "client",
            "connect/endpoints": ["tcp/127.0.0.1:7447"],
            "adminspace": {"enabled": True},
            "metadata": {"name": service_name},
        }

        config = zenoh.Config()
        for key, value in configuration.items():
            config.insert_json5(key, json.dumps(value))

        self.config = config


class ZenohRouter:
    prefix: str
    zenoh_session: ZenohSession
    service_name: str

    def __init__(self, service_name: str):
        self.service_name = service_name
        self.prefix = blueos_idl.http_gateway_prefix(service_name)
        self.zenoh_session = ZenohSession(service_name)

    def add_queryable(self, path: str, func: Callable[..., Any]) -> None:
        full_path = self.prefix
        if path:
            full_path += f"/{path}"

        def wrapper(query: zenoh.Query) -> None:
            params = dict(query.parameters)  # type: ignore

            async def _handle_async() -> None:
                try:
                    response = await func(**params)
                    if response is not None:
                        # REST-over-zenoh gateway: JSON, not IDL CDR (D-18).
                        query.reply(
                            query.selector.key_expr,
                            json.dumps(response, default=str),
                            encoding=HTTP_GATEWAY_JSON_ENCODING,
                        )
                except Exception as error:
                    logger.exception(f"Error in zenoh query handler: {query.selector.key_expr}")
                    error_response = {
                        "error": str(error),
                        "error_type": type(error).__name__,
                    }
                    query.reply(
                        query.selector.key_expr,
                        json.dumps(error_response),
                        encoding=HTTP_GATEWAY_JSON_ENCODING,
                    )

            def run_async() -> None:
                asyncio.run(_handle_async())

            self.zenoh_session.submit_to_executor(run_async)

        if self.zenoh_session.session:
            self.zenoh_session.session.declare_queryable(full_path, wrapper)

    def add_publisher(
        self,
        path: str,
        *,
        absolute: bool = False,
        publisher_options: dict[str, Any] | None = None,
    ) -> zenoh.Publisher | None:
        if absolute:
            full_path = path
        else:
            full_path = self.prefix
            if path:
                full_path += f"/{path}"

        if self.zenoh_session.session is None:
            return None

        return self.zenoh_session.session.declare_publisher(full_path, **(publisher_options or {}))

    def add_routes_to_zenoh(self, app: fastapi.FastAPI) -> None:
        queryables = []
        for route in app.router.routes:
            route_type = type(route)
            if (
                isinstance(route, APIRoute)
                and route_type.__name__ == "VersionedAPIRoute"
                and "fastapi_versioning" in route_type.__module__
                and "GET" in route.methods
            ):
                queryables.append((clean_path(route.path), route.endpoint))

        for path, func in queryables:
            self.add_queryable(path, func)


def clean_path(path: str) -> str:
    path = path.removeprefix("/").removesuffix("/")

    zenoh_path = re.sub(PARAM_REGEX, "*", path)
    zenoh_path = zenoh_path.replace("*/*", "**")

    return zenoh_path
