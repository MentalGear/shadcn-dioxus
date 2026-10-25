// The default base of select.mjs's diff (backlog row 418): the NEAREST of origin/main and every origin/queue/* ref.
//
// A lane cut from an unmerged merge-queue branch sees every queue commit as its own change against origin/main, so the
// selector would pick nearly the whole suite. Scoring each candidate by the commits HEAD has past their merge-base picks
// the branch the lane really grew from. Ties go to origin/main. Node only, reads the local refs, no network.
import { execFileSync } from "node:child_process";

const git = (cwd, ...a) => {
  try { return execFileSync("git", a, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim(); } catch { return null; }
};

// -> { ref, n }: n is the commits HEAD has past the merge-base with ref, or null when no candidate shares history with HEAD
export function pickBase(cwd, head = "HEAD") {
  const queued = (git(cwd, "for-each-ref", "--format=%(refname:short)", "refs/remotes/origin/queue") ?? "").split("\n").filter(Boolean);
  let best = null;
  for (const cand of ["origin/main", ...queued]) {
    const mb = git(cwd, "merge-base", cand, head);
    if (!mb) continue;
    const n = Number(git(cwd, "rev-list", "--count", `${mb}..${head}`));
    if (!Number.isFinite(n)) continue;
    if (!best || n < best.n || (n === best.n && cand === "origin/main")) best = { ref: cand, n };
  }
  return best ?? { ref: "origin/main", n: null };
}
