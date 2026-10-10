# NFSMW macOS patches

The working `main` branch combines the NFSMW patch branches with athei/mtld3d 1b0bf1f40c8bfeecb4b0c5a2c8182ef6893edd76. It contains the source changes used by the local NFSMW macOS preview on 2026-09-26. No game data is included.

## Immediate presentation pacing

Use an interruptible CPU deadline for an explicit frame cap when VSync is disabled. Avoid imposing the display refresh cadence through Metal minimum drawable duration in that mode. Preserve display-paced VSync. Unit tests cover cap cadence, late frames, uncapped operation and shutdown.

## Vertex spacing and read spans

Preserve a caller's nonzero vertex stride even if the consumed declaration extent is larger. Since the merge of upstream 0.12, upstream's crossing-attribute fetch (#1063) does this: an attribute that ends past its stream's stride is fetched through a binding of its own, and a staged buffer's recorded read range grows by what the last element reads past its stride (`crossing_read_size`). It replaced this fork's own read-span extension.

The fork's regression `stride_below_consumed_extent_preserves_fetch_and_prior_draw` in `windows/tests/tests/e2e/streams.rs` is kept beside upstream's crossing tests: it draws a triangle whose colour lies past a 12-byte stride, refills those colours in the same frame, and checks that the earlier draw keeps its colours.

## Verification and limits

The final source passed make check and make test ISOLATED=1 on Apple M1, macOS 27 beta: 1557 Windows host tests, 642 Unix host tests, and 881 end-to-end tests per Wine architecture. Each architecture reported 892 tests total, 11 ignored, zero failures and zero unaccounted tests. The production build and isolated installation passed with PROD=1 FP=1. See CONTRIBUTING.md for the build environment and commands.

A 120 FPS limit is supported. Sustained 120 FPS racing has not been achieved. These patches do not implement DLSS, temporal reconstruction, frame generation or ray tracing. The existing renderer provides MetalFX spatial scaling.

## Upstream refresh

The merged `nfsmw-upstream-update` history contains the retained-stencil correction, test-window shutdown correction and regression benchmark harness from upstream. Both NFSMW branch histories are preserved in `main`. This update has not been shown to increase game FPS.
