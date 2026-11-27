class ExtensionNotFound(Exception):
    pass


class ExtensionNotRunning(Exception):
    pass


class ExtensionPullFailed(Exception):
    pass


class ExtensionCleanupFailed(Exception):
    pass


class IncompatibleExtension(Exception):
    pass


class ExtensionInsufficientStorage(Exception):
    pass


class ExtensionInstallInProgress(Exception):
    pass
