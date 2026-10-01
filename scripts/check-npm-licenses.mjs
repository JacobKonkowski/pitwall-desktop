#!/usr/bin/env node
/**
 * Fail CI if npm dependency licenses fall outside the OSS allowlist.
 * DevDependencies are included (build/docs tools ship in contributor trees).
 */
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(path.join(root, "package.json"));

/** SPDX / license-checker strings we accept for this MIT app. */
const ALLOWED = new Set([
  "MIT",
  "ISC",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "0BSD",
  "BlueOak-1.0.0",
  "CC-BY-4.0",
  "CC-BY-3.0",
  "Python-2.0",
  "CC0-1.0",
  "Unlicense",
  "WTFPL",
  "MIT OR Apache-2.0",
  "Apache-2.0 OR MIT",
  "MIT AND ISC",
  "MIT AND Apache-2.0",
  "MIT AND CC-BY-3.0",
  "(MIT OR Apache-2.0)",
  "(Apache-2.0 OR MIT)",
  "(MIT AND CC-BY-3.0)",
]);

function normalizeLicense(raw) {
  if (raw == null) return "UNKNOWN";
  if (Array.isArray(raw)) return raw.map(String).join(" AND ");
  return String(raw).trim();
}

function isAllowed(license) {
  if (ALLOWED.has(license)) return true;
  const stripped = license.replace(/^\(|\)$/g, "");
  if (ALLOWED.has(stripped)) return true;
  // Dual / compound license forms
  for (const sep of [/\s+OR\s+/i, /\s+AND\s+/i]) {
    const parts = stripped.split(sep);
    if (parts.length > 1 && parts.every((p) => ALLOWED.has(p.trim().replace(/^\(|\)$/g, "")))) {
      return true;
    }
  }
  return false;
}

let data;
try {
  // Prefer programmatic API when license-checker is installed.
  const checker = require("license-checker");
  data = await new Promise((resolve, reject) => {
    checker.init(
      { start: root, production: false, excludePrivatePackages: true },
      (err, json) => (err ? reject(err) : resolve(json)),
    );
  });
} catch {
  // Fallback: npx (first CI run / no local install).
  const out = execFileSync(
    "npx",
    ["--yes", "license-checker", "--json", "--excludePrivatePackages"],
    { cwd: root, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  data = JSON.parse(out);
}

const violations = [];
for (const [pkg, info] of Object.entries(data)) {
  const license = normalizeLicense(info.licenses);
  if (!isAllowed(license)) {
    violations.push({ pkg, license, repository: info.repository || "" });
  }
}

if (violations.length) {
  console.error("npm license allowlist violations:\n");
  for (const v of violations.sort((a, b) => a.pkg.localeCompare(b.pkg))) {
    console.error(`  ${v.pkg}\n    license: ${v.license}\n    repo: ${v.repository}`);
  }
  console.error(
    `\nUpdate scripts/check-npm-licenses.mjs only after confirming redistribution is OK.\nSee docs/IP.md.`,
  );
  process.exit(1);
}

console.log(`npm licenses OK (${Object.keys(data).length} packages scanned).`);
