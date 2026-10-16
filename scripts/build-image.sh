#!/usr/bin/env bash
set -euo pipefail

for candidate in "$HOME/.cargo/bin" "/ucrt64/bin"; do
  case ":$PATH:" in
    *":$candidate:"*) ;;
    *) PATH="$candidate:$PATH";;
  esac
done
export PATH
command -v cargo gcc objcopy >/dev/null || { echo "missing toolchain" >&2; exit 3; }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUTDIR="target"
if [ "${1:-}" = "--output-dir" ]; then
  [ $# -ge 3 ] || { echo "usage: build-image.sh [--output-dir DIR] <normal|ud2|df|align>" >&2; exit 2; }
  OUTDIR="$2"
  shift 2
elif [[ "${1:-}" == --output-dir=* ]]; then
  OUTDIR="${1#--output-dir=}"
  shift
fi
[ $# -eq 1 ] || { echo "usage: build-image.sh [--output-dir DIR] <normal|ud2|df|align>" >&2; exit 2; }

VARIANT="$1"

OUTDIR="${OUTDIR#./}"
OUTDIR="$(printf '%s' "$OUTDIR" | sed -e 's|/*$||')"
[ -n "$OUTDIR" ] || { echo "invalid empty output directory" >&2; exit 2; }
case "$OUTDIR" in
  target/baseline-*|./target/baseline-*)
    echo "refusing to write into preserved baseline directory: $OUTDIR" >&2; exit 2;;
esac
mkdir -p "$OUTDIR" || { echo "cannot create output directory: $OUTDIR" >&2; exit 2; }
[ -d "$OUTDIR" ] || { echo "output directory is not a directory: $OUTDIR" >&2; exit 2; }

case "$VARIANT" in
  normal) CFG=""            ; TAG="v01"        ;;
  ud2)    CFG="--cfg ud2_test"   ; TAG="v01-ud2test"  ;;
  df)     CFG="--cfg df_test"    ; TAG="v01-dftest"   ;;
  align)  CFG="--cfg align_test" ; TAG="v01-aligntest";;
  *) echo "unknown variant: $VARIANT" >&2; exit 2;;
esac

KDIR="$OUTDIR/kernel-$TAG"
ELF="$KDIR/x86_64-unknown-none/release/okernel"
KBIN="$ELF.bin"
S2SRC="$OUTDIR/stage2-$TAG-gen.S"
SOBJ="$OUTDIR/stage2-$TAG.o"
SBIN="$OUTDIR/stage2-$TAG.bin"
SPAD="$OUTDIR/stage2-$TAG-4sector.bin"
S1OBJ="$OUTDIR/stage1-$TAG.o"
S1BIN="$OUTDIR/stage1-$TAG.bin"
KPAY="$OUTDIR/kernel-$TAG-64sector.bin"
IMG="$OUTDIR/okernel-$TAG.img"


echo "=== [$VARIANT] build kernel -> $ELF"
CARGO_TARGET_DIR="$KDIR" \
  cargo rustc --target x86_64-unknown-none --release -- \
    -C relocation-model=static -C link-arg=-Tkernel.ld \
    --check-cfg=cfg\(ud2_test\) --check-cfg=cfg\(df_test\) \
    --check-cfg=cfg\(align_test\) $CFG

echo "=== [$VARIANT] verify ELF"
ENTRY="$(objdump -f "$ELF" | grep -o 'start address 0x[0-9a-f]*')"
test "$ENTRY" = "start address 0x0000000000010000" || { echo "bad entry: $ENTRY" >&2; exit 1; }
objdump -t "$ELF" | grep -q "0000000000010000.*_start" || { echo "_start not at 0x10000" >&2; exit 1; }
if objdump -h "$ELF" | grep -q '\.bss.*[1-9]'; then
  : # .bss presence is fine; bounds checked below
fi
BSS_LINE="$(objdump -h "$ELF" | grep '\.bss' || true)"
if [ -n "$BSS_LINE" ]; then
  BSS_VMA="$(echo "$BSS_LINE" | awk '{print $4}')"
  BSS_SIZE="$(echo "$BSS_LINE" | awk '{print $3}')"
  BSS_END="$(printf '%x' "$((0x$BSS_VMA + 0x$BSS_SIZE))")"
  echo "bss: VMA 0x$BSS_VMA size 0x$BSS_SIZE end 0x$BSS_END"
  [ "$((0x$BSS_END))" -lt "$((0x17FF8))" ] || { echo ".bss overlaps sentinel" >&2; exit 1; }
fi

echo "=== [$VARIANT] export flat binary"
objcopy -O binary "$ELF" "$KBIN"
KSIZE="$(wc -c < "$KBIN" | tr -d ' ')"
[ "$KSIZE" -le 32768 ] || { echo "kernel too large: $KSIZE" >&2; exit 1; }
[ "$KSIZE" -ge 8 ] || { echo "kernel too small for signature: $KSIZE" >&2; exit 1; }
echo "kernel flat size: $KSIZE"

echo "=== [$VARIANT] derive signature from artifact"
HEX="$(od -An -tx1 -N8 "$KBIN" | tr -d ' \n')"
D0="0x${HEX:6:2}${HEX:4:2}${HEX:2:2}${HEX:0:2}"
D1="0x${HEX:14:2}${HEX:12:2}${HEX:10:2}${HEX:8:2}"
echo "kernel signature: $D0 $D1"

echo "=== [$VARIANT] test-behavior isolation"
UD2COUNT="$(od -An -tx1 "$KBIN" | tr -d ' \n' | grep -o '0f0b' | wc -l | tr -d ' ')"
echo "ud2 opcode count: $UD2COUNT"
case "$VARIANT" in
  normal|df|align)
    [ "$UD2COUNT" = "1" ] || { echo "unexpected ud2 count" >&2; exit 1; }
    ;;
  ud2)
    [ "$UD2COUNT" -ge 2 ] || { echo "deliberate ud2 path missing" >&2; exit 1; }
    ;;
esac
echo "isolation ok"

echo "=== [$VARIANT] stage per-variant stage2 source"
cp src/stage2.S "$S2SRC"
C0="$(grep -c -E 'cmpl \$0x[0-9a-fA-F]+, \(%si\)$' "$S2SRC")"
C1="$(grep -c -E 'cmpl \$0x[0-9a-fA-F]+, 4\(%si\)$' "$S2SRC")"
[ "$C0" = "1" ] || { echo "expected exactly one first-word signature check, found $C0" >&2; exit 1; }
[ "$C1" = "1" ] || { echo "expected exactly one second-word signature check, found $C1" >&2; exit 1; }
sed -E -i -e 's/cmpl \$0x[0-9a-fA-F]+, \(%si\)$/cmpl $'"$D0"', (%si)/' \
           -e 's/cmpl \$0x[0-9a-fA-F]+, 4\(%si\)$/cmpl $'"$D1"', 4(%si)/' "$S2SRC"
[ "$(grep -c -F "cmpl \$$D0, (%si)" "$S2SRC")" = "1" ] || { echo "first-word substitution failed" >&2; exit 1; }
[ "$(grep -c -F "cmpl \$$D1, 4(%si)" "$S2SRC")" = "1" ] || { echo "second-word substitution failed" >&2; exit 1; }
echo "staged: $S2SRC"

echo "=== [$VARIANT] build stage1 + stage2"
gcc -c src/stage1.S -o "$S1OBJ"
objcopy -O binary -j .text "$S1OBJ" "$S1BIN"
[ "$(wc -c < "$S1BIN" | tr -d ' ')" = "512" ] || { echo "stage1 not 512" >&2; exit 1; }
[ "$(od -An -tx1 -j510 -N2 "$S1BIN" | tr -d ' \n')" = "55aa" ] || { echo "stage1 sig bad" >&2; exit 1; }
gcc -c "$S2SRC" -o "$SOBJ"
objcopy -O binary -j .text "$SOBJ" "$SBIN"
[ "$(wc -c < "$SBIN" | tr -d ' ')" -le 2048 ] || { echo "stage2 too large" >&2; exit 1; }
D0LE="$(printf '%s' "$D0" | sed -E 's/^0x(..)(..)(..)(..)$/\4\3\2\1/')"
D1LE="$(printf '%s' "$D1" | sed -E 's/^0x(..)(..)(..)(..)$/\4\3\2\1/')"
S2FLAT="$(od -An -tx1 "$SBIN" | tr -d ' \n')"
case "$S2FLAT" in
  *"$D0LE"*) ;;
  *) echo "stage2 binary lacks signature $D0" >&2; exit 1;;
esac
case "$S2FLAT" in
  *"$D1LE"*) ;;
  *) echo "stage2 binary lacks signature $D1" >&2; exit 1;;
esac
echo "stage1+stage2 ok"

echo "=== [$VARIANT] package image"
dd if=/dev/zero of="$SPAD" bs=512 count=4 status=none
dd if="$SBIN" of="$SPAD" conv=notrunc status=none
dd if=/dev/zero of="$KPAY" bs=512 count=64 status=none
dd if="$KBIN" of="$KPAY" conv=notrunc status=none
printf 'OK64SEC!' | dd of="$KPAY" bs=1 seek=32760 conv=notrunc status=none
cat "$S1BIN" "$SPAD" "$KPAY" > "$IMG"

echo "=== [$VARIANT] verify image"
[ "$(wc -c < "$IMG" | tr -d ' ')" = "35328" ] || { echo "image size bad" >&2; exit 1; }
[ "$(od -An -tx1 -j510 -N2 "$IMG" | tr -d ' \n')" = "55aa" ] || { echo "boot sig bad" >&2; exit 1; }
[ "$(od -An -tx1 -j2560 -N8 "$IMG" | tr -d ' \n')" = "$HEX" ] || { echo "kernel head mismatch" >&2; exit 1; }
[ "$(od -An -tx1 -j35320 -N8 "$IMG" | tr -d ' \n')" = "4f4b363453454321" ] || { echo "sentinel bad" >&2; exit 1; }
S2CHECK="$OUTDIR/.stage2-check-$TAG.bin"
dd if="$IMG" of="$S2CHECK" bs=512 skip=1 count=4 status=none
S2EMB="$(od -An -tx1 "$S2CHECK" | tr -d ' \n')"
S2GEN="$(od -An -tx1 "$SPAD" | tr -d ' \n')"
[ "$S2EMB" = "$S2GEN" ] || { echo "embedded stage2 differs from generated payload" >&2; exit 1; }
rm -f "$S2CHECK"
echo "image ok: $IMG"