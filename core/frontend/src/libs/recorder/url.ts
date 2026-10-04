// eslint-disable-next-line import/prefer-default-export
export function recordingDownloadUrl(relativePath: string, httpPrefix: string): string {
  const encoded = relativePath.split('/').map((segment) => encodeURIComponent(segment)).join('/')
  return `${httpPrefix}/${encoded}`
}
