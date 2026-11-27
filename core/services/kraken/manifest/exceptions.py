from typing import Any, Dict, List


class ManifestDataFetchFailed(Exception):
    pass


class ManifestInvalidURL(Exception):
    pass


class ManifestDataParseFailed(Exception):
    pass


class ManifestNotFound(Exception):
    pass


class ManifestOperationNotAllowed(Exception):
    pass


class ManifestBackendOffline(Exception):
    pass


class ExtensionEntryNotFound(Exception):
    pass


class ExtensionSourceConflict(Exception):
    def __init__(self, identifier: str, installed_docker: str, candidates: List[Dict[str, Any]]) -> None:
        super().__init__(f"Extension {identifier} is installed from {installed_docker}, which no manifest source lists")
        self.identifier = identifier
        self.installed_docker = installed_docker
        self.candidates = candidates
