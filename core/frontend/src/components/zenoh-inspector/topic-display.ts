export function livelinessLabel(alive: boolean | undefined): string {
  if (alive === undefined) {
    return 'Unknown'
  }
  return alive ? 'Alive' : 'Dead'
}

export function livelinessColor(alive: boolean | undefined): string {
  if (alive === undefined) {
    return 'grey'
  }
  return alive ? 'success' : 'error'
}
