# Security policy

## Supported versions

Security fixes are provided for the latest published release and the current
`main` branch.

## Reporting a vulnerability

Do not open a public issue for vulnerabilities that could expose client media,
local paths, update signing material or checkpoint data. Use GitHub private
vulnerability reporting from the repository **Security** tab.

Include the affected version, operating system, reproduction steps and whether
real production media was involved. Do not attach client media.

## Data boundary

- Media analysis and offload are local by default.
- Crash reporting is opt-in and must scrub file names and paths.
- The source volume is opened read-only by the offload workflow.
- A successful checksum is not permission to format source media. Only the
  explicit `SAFE_TO_FORMAT` verdict has that meaning.

The complete fault/adversary boundary and residual risks are documented in
[`docs/threat-model.md`](docs/threat-model.md).
