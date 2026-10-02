// eslint-disable-next-line import/prefer-default-export
export function formatBandwidth(bytesPerSecond: number): string {
  const mbps = 8 * bytesPerSecond / 1024 / 1024
  let decimal_places = 0
  if (mbps < 10) {
    decimal_places = 2
  } else if (mbps < 100) {
    decimal_places = 1
  }
  return `${mbps.toFixed(decimal_places)}Mbps`
}
