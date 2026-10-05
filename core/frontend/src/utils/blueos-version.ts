const VERSION_PATH = '/blueos-version'
const DESCRIBE_PATTERN = /-\d+-g[0-9a-f]{7,}$/

let git_describe = ''
let build_date = ''

export function blueosGitDescribe(): string {
  return git_describe
}

export function blueosBuildDate(): string {
  return build_date
}

// First line is GIT_DESCRIBE_TAGS. Second line, when present, is the committer
// time of that commit (`git log -1 --format=%cI`).
export function parseBlueosVersion(body: string): { git_describe: string, build_date: string } {
  const [describe_line, date_line] = body.trim().split('\n')
  const parsed_date = date_line ? Date.parse(date_line) : Number.NaN
  return {
    git_describe: describe_line && DESCRIBE_PATTERN.test(describe_line) ? describe_line : '',
    build_date: Number.isNaN(parsed_date) ? '' : new Date(parsed_date).toLocaleString(),
  }
}

export async function loadBlueosVersion(): Promise<string> {
  try {
    const response = await fetch(VERSION_PATH, { cache: 'no-store', signal: AbortSignal.timeout(3000) })
    if (response.ok) {
      const parsed = parseBlueosVersion(await response.text())
      git_describe = parsed.git_describe
      build_date = parsed.build_date
    }
  } catch (error) {
    console.error('Failed to read BlueOS version', error)
  }
  return git_describe
}
