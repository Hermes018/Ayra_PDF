#!/bin/bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

echo "Fetching standard PDFs..."
wget -q -O standard-text.pdf https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf || curl -s -o standard-text.pdf https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf
wget -q -O standard-form.pdf https://www.irs.gov/pub/irs-pdf/fw4.pdf || curl -s -o standard-form.pdf https://www.irs.gov/pub/irs-pdf/fw4.pdf

echo "Generating synthetic 1,000 page PDF..."
# Create 1000 pages by combining dummy.pdf 1000 times
pages=""
for i in {1..10}; do
  pages="$pages standard-text.pdf"
done
qpdf --empty --deterministic-id --pages $pages -- 10_pages.pdf

pages_100=""
for i in {1..10}; do
  pages_100="$pages_100 10_pages.pdf"
done
qpdf --empty --deterministic-id --pages $pages_100 -- 100_pages.pdf

pages_1000=""
for i in {1..10}; do
  pages_1000="$pages_1000 100_pages.pdf"
done
qpdf --empty --deterministic-id --pages $pages_1000 -- synthetic-1k.pdf

rm 10_pages.pdf 100_pages.pdf

echo "Generating SHA256 hashes..."
sha256sum standard-text.pdf standard-form.pdf synthetic-1k.pdf > SHA256SUMS

echo "Verifying against MANIFEST..."
grep "sha256:" MANIFEST | while read -r line; do
  filename=$(echo "$line" | sed 's/^- \(.*\.pdf\).*/\1/')
  expected=$(echo "$line" | sed 's/.*sha256: \([a-f0-9]*\).*/\1/')
  actual=$(grep "$filename" SHA256SUMS | awk '{print $1}')
  
  if [ "$expected" == "$actual" ]; then
    echo "✅ $filename hash matches."
  else
    echo "❌ $filename hash mismatch! Expected $expected, got $actual"
    exit 1
  fi
done

echo "Done!"
