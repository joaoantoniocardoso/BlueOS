/* eslint-disable import/prefer-default-export */
export class BlueosApiError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'BlueosApiError'
  }
}
