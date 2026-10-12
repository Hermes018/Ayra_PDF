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

echo "Generating hostile test PDFs..."
printf "%%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n4 0 obj\n<< /Length 1073741824 /Filter /FlateDecode >>\nstream\nx\x9c\xec\xc1\x81\x00\x00\x00\x00\x80\xa0\xfd\xa3\x01\x00\x00\x00\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000201 00000 n \ntrailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n380\n%%%%EOF\n" > hostile-bombs.pdf

printf "%%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\nxref\n0 3\n0000000000 65535 f \n-000000001 00000 n \n9999999999 00000 n \ntrailer\n<< /Size 3 /Root 1 0 R /Prev 9999999999 >>\nstartxref\n120\n%%%%EOF\n" > hostile-broken-xref.pdf

echo "Generating SHA256 hashes..."
sha256sum standard-text.pdf standard-form.pdf synthetic-1k.pdf hostile-bombs.pdf hostile-broken-xref.pdf > SHA256SUMS


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
