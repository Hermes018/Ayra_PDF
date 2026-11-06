#!/bin/sh
# Builds the test corpus in this directory. PDFs are generated/fetched, never committed.
# POSIX sh (dash-safe) and idempotent: files whose hash already matches SHA256SUMS are kept,
# so a second run does no network access and changes nothing.
#
#   sh corpus/fetch.sh                 fetch/generate missing or stale files, then verify
#   sh corpus/fetch.sh --update-sums   regenerate SHA256SUMS from the current files (maintainers)
#
# Needs: wget or curl (first run only), qpdf, poppler-utils (pdftoppm, pdftotext, pdfinfo,
# pdfimages), python3, sha256sum, cmp.
#
# Two files are NOT in SHA256SUMS; each is verified by a host-independent content check instead,
# so a different poppler/qpdf/libjpeg build never hard-fails:
# - encrypted.pdf: qpdf AES-256 (R6) always draws a random file key, salts and /Perms block, so its
#   bytes differ on every run (no qpdf option fixes them). Check: R = 6, opens with lpdf-test,
#   5 pages, and its text (pdftotext) equals standard-form.pdf's text.
# - scanned.pdf: its pixels come from poppler's rasteriser and libjpeg, which differ across
#   versions. Check: 5 pages, no text layer, exactly one 850x1100 image per page, qpdf --check ok.
set -eu


DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$DIR"

# Cleanup trap AFTER the cd, with absolute paths: it must only ever touch this corpus directory.
TMPD=""
cleanup() {
  if [ -n "$TMPD" ]; then rm -rf "$TMPD"; fi
  rm -f "$DIR"/*.part "$DIR/10_pages.pdf" "$DIR/100_pages.pdf"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

UPDATE_SUMS=0
[ "${1:-}" = "--update-sums" ] && UPDATE_SUMS=1

ENC_USER_PW=lpdf-test
ENC_OWNER_PW=lpdf-owner

# expected_sum <file>: the hash recorded for <file> in SHA256SUMS (empty if none).
expected_sum() {
  [ -f SHA256SUMS ] || return 0
  awk -v f="$1" '$2 == f { print $1 }' SHA256SUMS
}

# is_current <file>: true when <file> exists and matches its SHA256SUMS entry.
is_current() {
  [ -f "$1" ] || return 1
  want=$(expected_sum "$1")
  [ -n "$want" ] || return 1
  have=$(sha256sum "$1" | awk '{ print $1 }')
  [ "$want" = "$have" ]
}

fetch() { # fetch <file> <url>
  if is_current "$1"; then echo "  $1: present"; return 0; fi
  echo "  $1: downloading"
  if command -v wget >/dev/null 2>&1; then
    wget -q -O "$1.part" "$2"
  else
    curl -fsSL -o "$1.part" "$2"
  fi
  mv "$1.part" "$1"
}

echo "Fetching standard PDFs..."
fetch standard-text.pdf https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf
fetch standard-form.pdf https://www.irs.gov/pub/irs-pdf/fw4.pdf

# replace_file <new> <target>: move <new> over <target> only when the content differs. A corpus
# file may be a symlink into the main checkout; mv replaces the link itself, never its target.
replace_file() {
  if [ -e "$2" ] && cmp -s "$1" "$2"; then
    rm -f "$1"
  else
    mv -f "$1" "$2"
  fi
}

# repeat <n> <word>: prints <word> <n> times, space separated.
repeat() {
  i=0; out=""
  while [ "$i" -lt "$1" ]; do out="$out $2"; i=$((i + 1)); done
  echo "$out"
}

echo "Generating synthetic 1,000 page PDF..."
if is_current synthetic-1k.pdf; then
  echo "  synthetic-1k.pdf: present"
else
  # shellcheck disable=SC2046 # word splitting of the repeated file list is intended
  qpdf --empty --deterministic-id --pages $(repeat 10 standard-text.pdf) -- 10_pages.pdf
  # shellcheck disable=SC2046
  qpdf --empty --deterministic-id --pages $(repeat 10 10_pages.pdf) -- 100_pages.pdf
  # shellcheck disable=SC2046
  qpdf --empty --deterministic-id --pages $(repeat 10 100_pages.pdf) -- synthetic-1k.pdf.part
  rm -f 10_pages.pdf 100_pages.pdf
  replace_file synthetic-1k.pdf.part synthetic-1k.pdf
fi

echo "Generating hostile test PDFs..."
# Octal escapes only: dash's printf has no \x.
printf "%%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n4 0 obj\n<< /Length 1073741824 /Filter /FlateDecode >>\nstream\nx\234\354\301\201\000\000\000\000\200\240\375\243\001\000\000\000\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000201 00000 n \ntrailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n380\n%%%%EOF\n" > hostile-bombs.pdf.part
replace_file hostile-bombs.pdf.part hostile-bombs.pdf

printf "%%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\nxref\n0 3\n0000000000 65535 f \n-000000001 00000 n \n9999999999 00000 n \ntrailer\n<< /Size 3 /Root 1 0 R /Prev 9999999999 >>\nstartxref\n120\n%%%%EOF\n" > hostile-broken-xref.pdf.part
replace_file hostile-broken-xref.pdf.part hostile-broken-xref.pdf

# scanned_ok: host-independent content check (see header).
scanned_ok() {
  [ -f scanned.pdf ] || return 1
  [ "$(pdfinfo scanned.pdf 2>/dev/null | awk '/^Pages:/ { print $2 }')" = 5 ] || return 1
  [ -z "$(pdftotext -q scanned.pdf - | tr -d ' \t\n\f')" ] || return 1
  [ "$(pdfimages -list scanned.pdf 2>/dev/null | awk 'NR > 2 && $3 == "image" && $4 == 850 && $5 == 1100 { print $1 }' | sort -u | wc -l)" -eq 5 ] || return 1
  [ "$(pdfimages -list scanned.pdf 2>/dev/null | awk 'NR > 2' | wc -l)" -eq 5 ] || return 1
  qpdf --check scanned.pdf >/dev/null 2>&1
}

echo "Generating scanned.pdf (image-only pages from standard-form.pdf)..."
if scanned_ok; then
  echo "  scanned.pdf: present"
else
  TMPD=$(mktemp -d "${TMPDIR:-/tmp}/lpdf-scan.XXXXXX")
  # 100 dpi JPEG per page (850x1100 px for US Letter); bytes depend on the poppler/libjpeg build.
  pdftoppm -r 100 -gray -jpeg -jpegopt quality=75 standard-form.pdf "$TMPD/page"
  # Our CLI (localpdf) has no image->PDF command, so a tiny stdlib-only writer wraps each JPEG
  # (DCTDecode, no re-encode) in a US-Letter page. No text layer, no dates/IDs: deterministic.
  python3 -I - "$TMPD" scanned.pdf.part <<'PY'
import os, sys, struct
src, out = sys.argv[1], sys.argv[2]
pages = sorted(f for f in os.listdir(src) if f.endswith(".jpg"))
def jpeg_info(b):
    i = 2
    while i < len(b):
        if b[i] != 0xFF: i += 1; continue
        m = b[i + 1]
        if m in (0xC0, 0xC1, 0xC2):
            h, w = struct.unpack(">HH", b[i + 5:i + 9]); return w, h, b[i + 9]
        i += 2 + struct.unpack(">H", b[i + 2:i + 4])[0]
    raise SystemExit("no SOF marker")
objs = []                      # object bodies, numbered from 1
def add(body): objs.append(body); return len(objs)
catalog = add(None); pages_obj = add(None); kids = []
for name in pages:
    data = open(os.path.join(src, name), "rb").read()
    w, h, ncomp = jpeg_info(data)
    cs = b"/DeviceGray" if ncomp == 1 else b"/DeviceRGB"
    img = add(b"<< /Type /XObject /Subtype /Image /Width %d /Height %d /ColorSpace %s "
              b"/BitsPerComponent 8 /Filter /DCTDecode /Length %d >>\nstream\n"
              % (w, h, cs, len(data)) + data + b"\nendstream")
    content = b"q 612 0 0 792 0 0 cm /Im0 Do Q"
    c = add(b"<< /Length %d >>\nstream\n" % len(content) + content + b"\nendstream")
    kids.append(add(b"<< /Type /Page /Parent %d 0 R /MediaBox [0 0 612 792] "
                    b"/Resources << /XObject << /Im0 %d 0 R >> >> /Contents %d 0 R >>"
                    % (pages_obj, img, c)))
objs[catalog - 1] = b"<< /Type /Catalog /Pages %d 0 R >>" % pages_obj
objs[pages_obj - 1] = (b"<< /Type /Pages /Kids [" + b" ".join(b"%d 0 R" % k for k in kids)
                       + b"] /Count %d >>" % len(kids))
buf = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"); offs = []
for n, body in enumerate(objs, 1):
    offs.append(len(buf)); buf += b"%d 0 obj\n" % n + body + b"\nendobj\n"
xref = len(buf)
buf += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1)
for o in offs: buf += b"%010d 00000 n \n" % o
buf += b"trailer\n<< /Size %d /Root %d 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objs) + 1, catalog, xref)
tmp = out + ".part"
open(tmp, "wb").write(buf); os.replace(tmp, out)
PY
  rm -rf "$TMPD"
  TMPD=""
  mv -f scanned.pdf.part scanned.pdf
fi

echo "Generating encrypted.pdf (AES-256, user password $ENC_USER_PW)..."
enc_ok() {
  [ -f encrypted.pdf ] || return 1
  qpdf --show-encryption --password="$ENC_USER_PW" encrypted.pdf 2>/dev/null | grep -q '^R = 6$' || return 1
  [ "$(pdfinfo -upw "$ENC_USER_PW" encrypted.pdf 2>/dev/null | awk '/^Pages:/ { print $2 }')" = 5 ] || return 1
  [ "$(pdftotext -q -upw "$ENC_USER_PW" encrypted.pdf - | sha256sum)" = "$(pdftotext -q standard-form.pdf - | sha256sum)" ]
}
if enc_ok; then
  echo "  encrypted.pdf: present"
else
  qpdf --encrypt --user-password="$ENC_USER_PW" --owner-password="$ENC_OWNER_PW" --bits=256 -- \
    standard-form.pdf encrypted.pdf.part
  mv encrypted.pdf.part encrypted.pdf
fi

SUMMED="standard-text.pdf standard-form.pdf synthetic-1k.pdf hostile-bombs.pdf hostile-broken-xref.pdf"
if [ "$UPDATE_SUMS" -eq 1 ]; then
  echo "Updating SHA256SUMS..."
  # shellcheck disable=SC2086
  sha256sum $SUMMED > SHA256SUMS
fi

echo "Verifying..."
sha256sum -c SHA256SUMS
# Every MANIFEST hash must agree with SHA256SUMS.
grep 'sha256: ' MANIFEST | while read -r line; do
  f=$(echo "$line" | sed 's/^- \([^ ]*\.pdf\).*/\1/')
  want=$(echo "$line" | sed 's/.*sha256: \([a-f0-9]*\).*/\1/')
  have=$(expected_sum "$f")
  if [ "$want" != "$have" ]; then
    echo "MANIFEST/SHA256SUMS mismatch for $f: MANIFEST $want, SHA256SUMS $have" >&2
    exit 1
  fi
done
if scanned_ok; then
  echo "scanned.pdf: OK (5 image-only pages, 850x1100 each, no text layer; content check)"
else
  echo "scanned.pdf: FAILED content check" >&2
  exit 1
fi
if enc_ok; then
  echo "encrypted.pdf: OK (AES-256 R6, opens with $ENC_USER_PW, text equals standard-form.pdf)"
else
  echo "encrypted.pdf: FAILED functional check" >&2
  exit 1
fi
echo "Done."
