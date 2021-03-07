#!/bin/bash

# Stop in case of error
set -e

cd "$(dirname "${BASH_SOURCE[0]}")"

source ../scripts/ansi_color.sh
enable_color

print_start() {
  COL="$ANSI_YELLOW"
  if [ "x$2" != "x" ]; then
    COL="$2"
  fi
  printf "${COL}${1}$ANSI_NOCOLOR\n"
}

gstart () {
  print_start "$@"
}
gend () {
  :
}

if [ -n "$CI" ]; then
  echo "INFO: set 'gstart' and 'gend' for CI"
  gstart () {
    printf '::group::'
    print_start "$@"
    SECONDS=0
  }

  gend () {
    duration=$SECONDS
    echo '::endgroup::'
    printf "${ANSI_GRAY}took $((duration / 60)) min $((duration % 60)) sec.${ANSI_NOCOLOR}\n"
  }
fi

#---

# The VESTS testsuite: compliance testsuite, from: https://github.com/nickg/vests.git 388250486a
_vests () {
  gstart "[GHDL - test] vests"
  cd vests

  if ./testsuite.sh > vests.log 2>&1 ; then
    printf "${ANSI_GREEN}Vests is OK$ANSI_NOCOLOR\n"
    wc -l vests.log
  else
    cat vests.log
    printf "${ANSI_RED}Vests failure$ANSI_NOCOLOR\n"
    failures=vests
  fi

  cd ..
  gend
  [ "$failures" = "" ] || exit 1
}

#---

if [ "x$GHDL" = "x" ]; then
  if [ "x$prefix" != "x" ]; then
    export GHDL="$prefix/bin/ghdl"
  elif [ "x$(command -v which)" != "x" ]; then
    export GHDL="$(which ghdl)"
  else
    printf "${ANSI_RED}error: GHDL environment variable is not defined${ANSI_NOCOLOR}\n"
    exit 1
  fi
fi

rm -f test_ok
failures=""
tests=

for opt; do
  shift
  case "$opt" in
      [a-z]*) tests="$tests $opt" ;;
      --) break ;;
      *) echo "$0: unknown option $opt"; exit 2 ;;
  esac
done

if [ "x$tests" = "x" ]; then tests="sanity pyunit gna vests synth vpi vhpi"; fi

echo "> tests: $tests"
echo "> args: $@"

# Run a testsuite
do_test() {
  case $1 in
    sanity|gna|synth|vpi|vhpi)
      gstart "[GHDL - test] $1"
      cd "$1"
      ../suite_driver.sh $@
      cd ..
      gend
      [ "$failures" = "" ] || exit 1
    ;;

    pyunit)
      # The Python Unit testsuite: regression testsuite for Python bindings to libghdl
      gstart "[GHDL - test] pyunit"
      PYTHONPATH=$(pwd)/.. ${PYTHON:-python3} -m pytest -rA pyunit
      gend
    ;;

    vests)
      _vests
    ;;
    *)
      printf "${ANSI_RED}$0: test name '$1' is unknown${ANSI_NOCOLOR}\n"
      exit 1;;
  esac
}

gstart "GHDL is: $GHDL"
$GHDL version
echo "REF: $($GHDL version ref)"
echo "HASH: $($GHDL version hash)"
gend

gstart "GHDL help"
$GHDL help
gend

for t in $tests; do do_test "$t"; done

printf "${ANSI_GREEN}[GHDL - test] SUCCESSFUL${ANSI_NOCOLOR}\n"
touch test_ok
