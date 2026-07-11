#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const dir = resolve(here, "../src/locales");
const en = JSON.parse(readFileSync(resolve(dir, "en.json"), "utf8"));
const es = JSON.parse(readFileSync(resolve(dir, "es.json"), "utf8"));
const enK = new Set(Object.keys(en));
const esK = new Set(Object.keys(es));
const m1 = [...enK].filter((k) => !esK.has(k));
const m2 = [...esK].filter((k) => !enK.has(k));
if (m1.length || m2.length) {
  console.error("i18n gate FAILED");
  if (m1.length) console.error(" missing in es:", m1);
  if (m2.length) console.error(" missing in en:", m2);
  process.exit(1);
}
console.log("i18n gate OK");