#!/usr/bin/env node
// CI gate: every key in en.json must exist in es.json and vice versa.
// Per CLAUDE.md rule 8 + build README non-negotiable.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const localesDir = resolve(here, "../src/locales");

function load(name) {
  return JSON.parse(readFileSync(resolve(localesDir, `${name}.json`), "utf8"));
}

const en = load("en");
const es = load("es");
const enKeys = new Set(Object.keys(en));
const esKeys = new Set(Object.keys(es));

const missingInEs = [...enKeys].filter((k) => !esKeys.has(k));
const missingInEn = [...esKeys].filter((k) => !enKeys.has(k));

if (missingInEs.length || missingInEn.length) {
  console.error("i18n gate FAILED");
  if (missingInEs.length) console.error("  missing in es:", missingInEs);
  if (missingInEn.length) console.error("  missing in en:", missingInEn);
  process.exit(1);
}
console.log("i18n gate OK");