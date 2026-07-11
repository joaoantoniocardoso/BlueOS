#!/usr/bin/env bash
# Sample a BlueOS in-container process's CPU + RSS over a window and emit a JSON
# distribution (mean/median/p95/min/max/sd). CPU is top-style (100 == 1 core),
# derived from /proc/<pid>/stat vs /proc/stat deltas. Reusable across capture runs.
#
# Usage:
#   sample_resource.sh --match <proc-substr> [options]
# Options:
#   --host H (192.168.0.177)  --user U (pi)  --pass P (raspberry)
#   --container C (blueos-core)  --samples N (60)  --interval S (1)
#   --label L (resting)  --out FILE (stdout if unset)
set -euo pipefail

HOST=192.168.0.177; USER=pi; PASS=raspberry; CONTAINER=blueos-core
MATCH=""; SAMPLES=60; INTERVAL=1; LABEL=resting; OUT=""

while [ $# -gt 0 ]; do
  case "$1" in
    --host) HOST="$2"; shift 2;; --user) USER="$2"; shift 2;; --pass) PASS="$2"; shift 2;;
    --container) CONTAINER="$2"; shift 2;; --match) MATCH="$2"; shift 2;;
    --samples) SAMPLES="$2"; shift 2;; --interval) INTERVAL="$2"; shift 2;;
    --label) LABEL="$2"; shift 2;; --out) OUT="$2"; shift 2;;
    -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "unknown arg: $1" >&2; exit 1;;
  esac
done
[ -n "$MATCH" ] || { echo "error: --match is required" >&2; exit 1; }

raw=$(sshpass -p "$PASS" ssh -o StrictHostKeyChecking=no -o ConnectTimeout=8 "$USER@$HOST" \
  "docker exec -i $CONTAINER sh -s '$MATCH' '$SAMPLES' '$INTERVAL'" <<'REMOTE'
match="$1"; samples="$2"; interval="$3"
pid=$(ps -eo pid,rss,args | grep python3 | grep -F -- "$match" | grep -v grep | sort -k2 -nr | awk '{print $1}' | head -1)
[ -n "$pid" ] || { echo "ERR no matching python3 process for: $match" >&2; exit 3; }
ncpu=$(nproc)
echo "META $pid $ncpu $(cat /proc/$pid/comm)"
prev=$(awk '{print $14+$15}' /proc/$pid/stat)
ptot=$(awk 'NR==1{s=0;for(i=2;i<=NF;i++)s+=$i;print s}' /proc/stat)
i=0
while [ "$i" -lt "$samples" ]; do
  sleep "$interval"
  cur=$(awk '{print $14+$15}' /proc/$pid/stat 2>/dev/null) || break
  ctot=$(awk 'NR==1{s=0;for(i=2;i<=NF;i++)s+=$i;print s}' /proc/stat)
  rss=$(awk '/VmRSS/{print $2}' /proc/$pid/status)
  dp=$((cur-prev)); dt=$((ctot-ptot))
  cpu=$(awk -v dp="$dp" -v dt="$dt" -v n="$ncpu" 'BEGIN{if(dt>0)printf "%.2f",100.0*dp/dt*n; else print 0}')
  echo "S $cpu $rss"
  prev=$cur; ptot=$ctot
  i=$((i+1))
done
REMOTE
)

meta=$(printf '%s\n' "$raw" | awk '/^META/{print $2, $3, $4}')
pid=$(echo "$meta" | awk '{print $1}'); ncpu=$(echo "$meta" | awk '{print $2}'); comm=$(echo "$meta" | awk '{print $3}')

json=$(printf '%s\n' "$raw" | awk -v label="$LABEL" -v pid="$pid" -v ncpu="$ncpu" -v comm="$comm" \
  -v samples="$SAMPLES" -v interval="$INTERVAL" -v pat="$MATCH" '
  function pct(a,n,p,  idx){idx=int(n*p); if(idx<1)idx=1; if(idx>n)idx=n; return a[idx]}
  /^S /{c[++nc]=$2+0; r[nr+1]=$3+0; nr++}
  END{
    if(nc==0){print "{\"error\":\"no samples\"}"; exit}
    for(i=1;i<=nc;i++){for(j=i+1;j<=nc;j++){if(c[j]<c[i]){t=c[i];c[i]=c[j];c[j]=t} if(r[j]<r[i]){t=r[i];r[i]=r[j];r[j]=t}}}
    for(i=1;i<=nc;i++){sc+=c[i]; sr+=r[i]}
    mc=sc/nc; mr=sr/nc;
    for(i=1;i<=nc;i++){vc+=(c[i]-mc)^2}; sd=sqrt(vc/nc);
    printf "{\n";
    printf "  \"label\": \"%s\",\n", label;
    printf "  \"match\": \"%s\",\n", pat;
    printf "  \"pid\": %s, \"ncpu\": %s, \"comm\": \"%s\",\n", pid, ncpu, comm;
    printf "  \"samples\": %d, \"interval_s\": %s,\n", nc, interval;
    printf "  \"method\": \"/proc stat deltas, top-style (100 == 1 core)\",\n";
    printf "  \"cpu_pct\": {\"mean\": %.2f, \"median\": %.2f, \"p95\": %.2f, \"min\": %.2f, \"max\": %.2f, \"sd\": %.2f},\n", mc, pct(c,nc,0.5), pct(c,nc,0.95), c[1], c[nc], sd;
    printf "  \"rss_mb\": {\"mean\": %.1f, \"median\": %.1f, \"min\": %.1f, \"max\": %.1f}\n", mr/1024, pct(r,nc,0.5)/1024, r[1]/1024, r[nc]/1024;
    printf "}\n";
  }')

if [ -n "$OUT" ]; then printf '%s\n' "$json" > "$OUT"; echo "wrote $OUT" >&2; else printf '%s\n' "$json"; fi
