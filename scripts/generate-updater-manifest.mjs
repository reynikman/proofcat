#!/usr/bin/env node
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname } from "node:path";

const usage = `Usage:
  node scripts/generate-updater-manifest.mjs \\
    --repository OWNER/REPO --version X.Y.Z --tag vX.Y.Z \\
    --macos-signature path/to/ProofCat.app.tar.gz.sig \\
    --windows-signature path/to/ProofCat_x64-setup.exe.sig \\
    --output path/to/latest.json [--pub-date RFC3339] [--notes TEXT]`;

function fail(message) {
  console.error(`error: ${message}`);
  process.exit(1);
}

function parseArgs(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      console.log(usage);
      process.exit(0);
    }
    if (!arg.startsWith("--")) fail(`unexpected argument: ${arg}`);
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) fail(`missing value for ${arg}`);
    options[arg.slice(2)] = value;
    index += 1;
  }
  return options;
}

function requireOption(options, name) {
  if (!options[name]) fail(`--${name} is required`);
  return options[name];
}

function publicKeyId(configPath) {
  const config = JSON.parse(readFileSync(configPath, "utf8"));
  const encodedKey = config?.plugins?.updater?.pubkey;
  if (typeof encodedKey !== "string") fail(`${configPath} has no updater public key`);
  const lines = Buffer.from(encodedKey, "base64").toString("utf8").trim().split("\n");
  if (lines.length < 2) fail("updater public key is not a minisign key");
  const key = Buffer.from(lines[1], "base64");
  if (key.length < 10) fail("updater public key is malformed");
  return key.subarray(2, 10);
}

function readSignature(signaturePath, expectedKeyId) {
  const encoded = readFileSync(signaturePath, "utf8").trim();
  const lines = Buffer.from(encoded, "base64").toString("utf8").trim().split("\n");
  if (lines.length < 2 || !lines[0].startsWith("untrusted comment:")) {
    fail(`${signaturePath} is not a Tauri/minisign signature`);
  }
  const signature = Buffer.from(lines[1], "base64");
  if (signature.length < 10 || !signature.subarray(2, 10).equals(expectedKeyId)) {
    fail(`${signaturePath} was not made by the updater public key in tauri.conf.json`);
  }
  return encoded;
}

const options = parseArgs(process.argv.slice(2));
const repository = requireOption(options, "repository");
const version = requireOption(options, "version");
const tag = options.tag || `v${version}`;
const output = requireOption(options, "output");
const macosSignaturePath = requireOption(options, "macos-signature");
const windowsSignaturePath = requireOption(options, "windows-signature");
const pubDate = options["pub-date"] || new Date().toISOString();
const notes = options.notes || `ProofCat ${version}`;
const configPath = options.config || "src-tauri/tauri.conf.json";

if (!/^[^/\s]+\/[^/\s]+$/.test(repository)) fail("--repository must be OWNER/REPO");
if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
  fail("--version must be a SemVer version without a leading v");
}
if (Number.isNaN(Date.parse(pubDate))) fail("--pub-date must be an RFC3339-compatible date");

const keyId = publicKeyId(configPath);
const macosSignature = readSignature(macosSignaturePath, keyId);
const windowsSignature = readSignature(windowsSignaturePath, keyId);
const downloadBase = `https://github.com/${repository}/releases/download/${tag}`;

const manifest = {
  version,
  notes,
  pub_date: new Date(pubDate).toISOString(),
  platforms: {
    "darwin-aarch64": {
      url: `${downloadBase}/${basename(macosSignaturePath).replace(/\.sig$/, "")}`,
      signature: macosSignature,
    },
    "windows-x86_64": {
      url: `${downloadBase}/${basename(windowsSignaturePath).replace(/\.sig$/, "")}`,
      signature: windowsSignature,
    },
  },
};

mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
console.log(`wrote ${output}`);
