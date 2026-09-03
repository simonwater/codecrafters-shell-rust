#!/usr/bin/env bash

# 原有tester运行时需要传递json格式的关卡slug，想要验证的用例较多时比较麻烦。
# 此脚本的作用是支持类似官方线上的 --previous 模式，指定一个关卡，然后生成该关卡及以前所有关卡的json，再运行tester

# 指向你的 Go Tester 二进制文件路径（如果在当前目录下，直接填 ./tester）
TESTER_BIN="./tester_mac_x86.out"

# 按顺序定义 Shell 教程的所有 Stage Slug
STAGES=(
  "oo8" "cz2" "ff0" "pn5" "iz3" "ez5" "mg5" "ip1" # Base (1-8)
  "ei0" "ra6" "gq9" "gp4"                         # Navigation (9-12)
  "ni6" "tg6" "yt5" "le5" "gu3" "qj0"             # Quoting (13-18)
  "jv1" "vz4" "el9" "un3"                         # Redirections (19-22)
  "qp2" "gm9" "qm8" "gy5" "wh6" "wt6"             # Completions (23-28)
  "zv2" "ue6" "lc6" "vs5" "no5" "jp8" "bf8"       # Filename Completion (29-35)
  "ne7" "oi7" "wl6" "pm5" "qf1" "zi0" "nr7" "ep2" "xz3" "tz2" # Programmable (36-45)
  "br6" "ny9" "xk3"                               # Pipeline (46-48)
  "af3" "at7" "si2" "jd6" "dk5" "ma9" "rq2" "bv8" "fy4" # Background Jobs (49-57)
  "bq4" "yf5" "ag6" "rh7" "vq0" "dm2"             # History (58-63)
  "za2" "in3" "sx3" "zp4" "kz7" "jv2"             # History Persistence (64-69)
  "ji0" "oa2" "kv5" "db8" "ge9" "br2" "my0"       # Parameter Expansion (70-76)
)

TARGET_SLUG=$1
PREVIOUS=false

if [ "$2" == "--previous" ] || [ "$2" == "-p" ]; then
  PREVIOUS=true
fi

if [ -z "$TARGET_SLUG" ]; then
  echo "Usage: ./test.sh <stage_slug> [--previous|-p]"
  echo "Example: ./test.sh xk3 --previous"
  exit 1
fi

# 寻找目标 slug 在数组中的位置
TARGET_INDEX=-1
for i in "${!STAGES[@]}"; do
   if [[ "${STAGES[$i]}" == "${TARGET_SLUG}" ]]; then
       TARGET_INDEX=$i
       break
   fi
done

if [ $TARGET_INDEX -eq -1 ]; then
  echo "Error: Stage slug '${TARGET_SLUG}' not found."
  exit 1
fi

# 构建 JSON 关卡列表 (补全 title 字段)
JSON_STAGES="["
if [ "$PREVIOUS" = true ]; then
  # 包含从 0 到 TARGET_INDEX 的所有关卡
  for (( i=0; i<=$TARGET_INDEX; i++ )); do
    SLUG="${STAGES[$i]}"
    JSON_STAGES="${JSON_STAGES}{\"slug\":\"${SLUG}\",\"tester_log_prefix\":\"tester::#${SLUG}\",\"title\":\"Stage ${SLUG}\"}"
    if [ $i -lt $TARGET_INDEX ]; then
      JSON_STAGES="${JSON_STAGES},"
    fi
  done
else
  # 仅运行当前目标关卡
  JSON_STAGES="${JSON_STAGES}{\"slug\":\"${TARGET_SLUG}\",\"tester_log_prefix\":\"tester::#${TARGET_SLUG}\",\"title\":\"Stage ${TARGET_SLUG}\"}"
fi
JSON_STAGES="${JSON_STAGES}]"

# 禁用 rustup 自动更新并提前编译一次
export RUSTUP_AUTO_SELF_UPDATE=0
cargo build --quiet

# 执行测试
CODECRAFTERS_REPOSITORY_DIR="$(pwd)" \
CODECRAFTERS_TEST_CASES_JSON="${JSON_STAGES}" \
$TESTER_BIN