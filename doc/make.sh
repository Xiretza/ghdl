#!/usr/bin/env sh

set -e

cd "$(dirname $0)"

docker build -t ghdl/sphinx -f- . <<EOF
FROM btdi/sphinx:py3-featured
COPY requirements.txt /
RUN apk add -U --no-cache make \
 && pip3 install -r /requirements.txt
EOF

dcmd="docker run --rm -u $(id -u) -v /$(pwd)/..://tmp/src -w //tmp/src/doc"

$dcmd ghdl/sphinx sh -c "make html latex man"

set +e

$dcmd btdi/latex:latest bash -c "
cd _build/latex
#make
pdflatex -interaction=nonstopmode GHDL.tex;
makeindex -s python.ist GHDL.idx;
pdflatex -interaction=nonstopmode GHDL.tex;
"

set -e
