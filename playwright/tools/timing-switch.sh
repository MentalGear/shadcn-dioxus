# The one switch of the timing pass (sourced by tools/timing-pass.sh, tools/run-suite.sh and scripts/deploy-preview.sh).
#
# While it is `off`, the `@timing` cases do not run in the suite or in `test:changed`, and the deploy's verdict does not
# count a missing chunk-timing.json as red. Re-enabling is this one line: set the default to `on`, in the commit that
# closes the backlog row named below. An explicit `TIMING_PASS=on` in the environment runs the pass by hand either way.
# The tag, tools/timing-pass.sh, tools/wait-quiet.sh and scripts/check-timing-tags.sh stay as they are.
TIMING_PASS_DEFAULT=off
# The open backlog row that re-enables the pass (scripts/check-timing-tags.sh fails while this row is not open).
TIMING_REENABLE_ROW=494
TIMING_PASS="${TIMING_PASS:-$TIMING_PASS_DEFAULT}"
export TIMING_PASS TIMING_REENABLE_ROW
