# Fault matrix

Every strong verdict is covered by an automated fault or property test. The
public release record adds physical-media results to this matrix.

| Fault | Expected result | Automated evidence |
|---|---|---|
| Destination byte changed after write | repair only that replica; otherwise `FAILED` | `archive_max_repairs_destination_corrupted_before_readback` |
| Destination filesystem is full | New job: fail before the first media byte. Resume: check each remaining file before opening its writer, so already verified files do not block recovery. No insufficient file is written and no verdict is issued. | `disk_full_preflight_bails`, `test_per_file_space_gate_rejects_an_unfulfillable_write`, `scripts/macos-disk-full.sh` |
| Source changes between passes | `FAILED`; no verified replica | `archive_max_rejects_source_changed_between_reads` |
| Source disappears during repair | use only another hash-matched replica | `archive_max_repairs_from_verified_replica_when_source_disappears` |
| Crash/cancel in scan, pre-read, copy, readback, repair or MHL | same job resumes; final file is complete and independently verified | `scripts/process-kill-resume.sh`, `resume_converges_to_same_final_replica_evidence_from_each_interruptible_phase` |
| Resume with changed card/tree | stop before copy | `resume_context_rejects_replaced_media` |
| Same-size wrong destination | overwrite/verify, never blind skip | `same_size_different_content_overwrites` |
| Missing/truncated/tampered media | verifier fails | `reports_missing_file_without_a_false_pass`, `reports_truncated_file_as_hash_mismatch`, `silent_corruption_caught_by_mhl_verify` |
| Tampered MHL generation | chain failure | `test_chain_tamper_detection` |
| Unknown MHL hash | explicit unsupported error | `rejects_unknown_manifest_hash_algorithm`, `rejects_unknown_chain_reference_algorithm` |
| Unsafe path/symlink/case collision | block or skip before copy | `symlinks_skipped_not_followed`, `rejects_cross_platform_unsafe_names_before_copy`, `unsafe_source_name_blocks_before_any_destination_write` |
| Unicode and random trees | byte/MHL roundtrip | `unicode_emoji_long_names_roundtrip`, `random_tree_offload_mhl_roundtrip` |
| Manual verification cancel | stop hashing | `manual_verification_honors_cancel` |
| Source or destination disappears | no replica becomes verified without matching readback | `archive_max_source_loss_during_copy_never_verifies_replica`, `archive_max_destination_loss_exhausts_repairs_without_false_verified_state` |
| Disk image or ambiguous device | never contributes to `SAFE_TO_FORMAT` | `macos_disk_image_never_counts_as_physical_destination`, `safe_to_format_requires_every_gate` |
| Resume after source replacement | block before copy | `archive_max_resume_rejects_mutated_source_snapshot` |
| Zero-byte media | preserve and verify, but require operator review | `archive_max_zero_byte_media_requires_operator_review` |
| Parallel paths alias one disk | share one scheduler queue; no false concurrency | `aliases_on_one_physical_device_share_the_same_queue` |
| SSD small-file concurrency | bounded 1..4 effective workers, complete MHL | `fast_parallel_small_file_copy_is_ssd_only_bounded_and_complete` |

Recorded on real media for the v0.3.0 release: cable disconnect/reconnect
(during copy and read-back), real disk-full, wrong physical disk at resume, the
macOS/Windows filesystem matrix (including real NTFS) and the 24-hour soak. See
[`TEST_REPORT.md`](TEST_REPORT.md) (macOS matrix + Windows W1–W6 + soak) and the
exact-SHA provenance in [`release-evidence-v0.3.0.md`](release-evidence-v0.3.0.md).
The 100 GB benchmark remains a known follow-up.
