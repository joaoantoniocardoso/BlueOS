import type { Log } from '@blueos-idl/messages'
import { Encoding } from '@eclipse-zenoh/zenoh-ts'

import {
  encodeCdr,
  LOG_SCHEMA,
  logKey,
} from '@/libs/blueos-api'
import zenoh from '@/libs/zenoh'
import frontend from '@/store/frontend'

/**
 * Compliant with FoxGlove LogLevel
 * https://docs.foxglove.dev/docs/visualization/message-schemas/log-level
 */
enum LogLevel {
  UNKNOWN = 0,
  DEBUG = 1,
  INFO = 2,
  WARNING = 3,
  ERROR = 4,
  FATAL = 5,
}

class ConsoleLogger {
  private session: Awaited<ReturnType<typeof zenoh.getSession>> | null = null

  private static readonly STACK_LINE_REGEX = /\(?([^\s()]+):(\d+):\d+\)?/

  private static readonly LOG_ENCODING = Encoding.APPLICATION_CDR.withSchema(LOG_SCHEMA)

  readonly originalConsole: {
    log: typeof console.log
    info: typeof console.info
    warn: typeof console.warn
    error: typeof console.error
    debug: typeof console.debug
  }

  constructor() {
    this.originalConsole = {
      log: console.log,
      info: console.info,
      warn: console.warn,
      error: console.error,
      debug: console.debug,
    }
  }

  async initialize(): Promise<void> {
    if (this.session !== null) {
      return
    }

    try {
      this.session = await zenoh.getSession()
      this.interceptConsole()
      this.interceptWindow()
    } catch (error) {
      console.error('[ConsoleLogger] Failed to initialize:', error)
    }
  }

  private readonly onError = (event: ErrorEvent): void => {
    this.publishMessage(
      LogLevel.ERROR,
      [event.message, event.filename, event.lineno, event.colno, event.error],
      event.filename,
      event.lineno,
    )
  }

  private readonly onUnhandledRejection = (event: PromiseRejectionEvent): void => {
    this.publishMessage(LogLevel.ERROR, [event.reason, event.promise, event.type])
  }

  private interceptWindow(): void {
    window.addEventListener('error', this.onError)
    window.addEventListener('unhandledrejection', this.onUnhandledRejection)
  }

  private interceptConsole(): void {
    console.log = (...args: unknown[]) => {
      this.originalConsole.log(...args)
      this.publishMessage(LogLevel.INFO, args)
    }

    console.info = (...args: unknown[]) => {
      this.originalConsole.info(...args)
      this.publishMessage(LogLevel.INFO, args)
    }

    console.warn = (...args: unknown[]) => {
      this.originalConsole.warn(...args)
      this.publishMessage(LogLevel.WARNING, args)
    }

    console.error = (...args: unknown[]) => {
      this.originalConsole.error(...args)
      this.publishMessage(LogLevel.ERROR, args)
    }

    console.debug = (...args: unknown[]) => {
      this.originalConsole.debug(...args)
      this.publishMessage(LogLevel.DEBUG, args)
    }
  }

  private publishMessage(level: LogLevel, args: unknown[], file?: string, line?: number): void {
    if (!this.session) {
      return
    }

    try {
      const now = new Date()
      const timestamp = {
        sec: Math.floor(now.getTime() / 1000),
        nsec: now.getTime() % 1000 * 1000000,
      }

      if (file === undefined || line === undefined) {
        const { file: errorFile, line: errorLine } = ConsoleLogger.extractErrorLocation(args)
        file = errorFile
        line = errorLine
      }

      const message: Log = {
        timestamp,
        level,
        message: args.map((arg) => ConsoleLogger.stringifyArgument(arg)).join(' '),
        name: frontend.frontend_id,
        file: file ?? '',
        line: line ?? 0,
      }

      const topic = logKey('frontend')
      const payload = encodeCdr(LOG_SCHEMA, message)

      // put() is async in zenoh 1.9; swallow rejections via `originalConsole` so a failed publish
      // cannot surface as an `unhandledrejection` and re-enter `publishMessage` (infinite feedback loop).
      this.session.put(topic, payload, { encoding: ConsoleLogger.LOG_ENCODING })
        .catch((publishError) => {
          this.originalConsole.error('[ConsoleLogger] Failed to publish message:', publishError)
        })
    } catch (error) {
      this.originalConsole.error('[ConsoleLogger] Failed to publish message:', error)
    }
  }

  private static extractErrorLocation(args: unknown[]): { file: string | undefined; line: number | undefined } {
    for (const arg of args) {
      try {
        if (arg instanceof Error && typeof arg.stack === 'string') {
          const lines = arg.stack.split('\n')
          for (const lineText of lines) {
            const match = ConsoleLogger.STACK_LINE_REGEX.exec(lineText)
            if (match) {
              return {
                file: match[1],
                line: parseInt(match[2], 10),
              }
            }
          }
        }
      } catch {
        continue
      }
    }

    return { file: undefined, line: undefined }
  }

  private static stringifyArgument(arg: unknown): string {
    if (arg === null) {
      return 'null'
    }
    if (arg === undefined) {
      return 'undefined'
    }

    if (arg instanceof Error) {
      return `${arg.name}: ${arg.message}`
    }

    switch (typeof arg) {
      case 'string':
        return arg
      case 'boolean':
      case 'number':
        return String(arg)
      case 'bigint':
        return `${arg}n`
      case 'object':
        try {
          return JSON.stringify(arg)
        } catch {
          return '[Object]'
        }
      case 'function':
        return '[Function]'
      default:
        try {
          return String(arg)
        } catch {
          return '[Unknown]'
        }
    }
  }

  async cleanup(): Promise<void> {
    console.log = this.originalConsole.log
    console.info = this.originalConsole.info
    console.warn = this.originalConsole.warn
    console.error = this.originalConsole.error
    console.debug = this.originalConsole.debug

    window.removeEventListener('error', this.onError)
    window.removeEventListener('unhandledrejection', this.onUnhandledRejection)
    this.session = null
  }
}

const consoleLogger = new ConsoleLogger()

export default consoleLogger
